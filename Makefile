CARGO_WATCH_IGNORES := $(shell grep -E '^[^\#]| ?\n' .gitignore | sed 's/^/--ignore /')

.PHONY: all
all: 
	@$(MAKE) BIN=docker bin-exists
	@docker network create panya_net 2>/dev/null || true
	-@docker compose up -d --remove-orphans || true
	sudo -H -u mkd cargo install --path . --force
	# @docker run --rm --volume `pwd`/dist/panya:/usr/src/app/target/debug panya

.PHONY: build
build:
	@docker build -t panya -f build.Dockerfile .
	@docker run --rm --volume `pwd`/dist/panya:/usr/src/app/target/debug panya

.PHONY: scp-bin
scp-bin:
	scp -r ./config mkd@4thehoard.com:/home/mkd/.cargo/bin
	scp ./dist/panya/panya mkd@4thehoard.com:/home/mkd/.cargo/bin

.PHONY: restart-remote-panya
restart-remote-panya:
	ssh mkd@$4thehoard.com "sudo systemctl restart panya"

.PHONY: dev
dev: setup start

.PHONY: start
start:
	@docker compose up -d --remove-orphans
	$(MAKE) watch

.PHONY: watch
watch:
	RUST_BACKTRACE=1 cargo watch $(CARGO_WATCH_IGNORES) -x 'run'

.PHONY: setup
setup:
	@$(MAKE) BIN=cargo bin-exists
	@$(MAKE) BIN=docker bin-exists
	@cargo install cargo-watch

.PHONY: bin-exists
bin-exists:
	@if [ -z $(shell which $(BIN)) ] || [ ! -x $(shell which $(BIN)) ]; then \
		echo "Error: $(BIN) is not installed or not executable"; \
		exit 1; \
	fi

.PHONY: lint
lint:
	cargo fmt
	cargo clippy

.PHONY: test
test:
	cargo test

.PHONY: ci
ci: lint test