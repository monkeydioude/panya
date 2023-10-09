CARGO_WATCH_IGNORES := $(shell grep -E '^[^\#]| ?\n' .gitignore | sed 's/^/--ignore /')

.PHONY: all
all: 
	@$(MAKE) BIN=docker bin-exists
	@docker network create panya_net || echo "Network 'panya_net' already exists. Skipping."
	@docker compose up -d
	@docker build --add-host=deb.debian.org:199.232.170.132 -t panya .
	@$(MAKE) run-container

.PHONY: run-container
run-container:
	docker run -p 8083:8083 --network panya_net --name panya-api --rm panya

.PHONY: dev
dev: setup start

.PHONY: start
start:
	@docker compose up -d
	cargo watch $(CARGO_WATCH_IGNORES) -x 'run'

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