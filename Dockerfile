FROM rustlang/rust:nightly-bullseye-slim as builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get -y install pkg-config libssl-dev
COPY src ./src/
COPY Cargo.lock Cargo.toml Makefile ./
COPY config/docker.json ./config/default.json
RUN cargo build

FROM debian:bullseye
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/config/ /usr/local/bin/config
COPY --from=builder /usr/src/app/target/debug/panya /usr/local/bin/app
CMD ["app"]
