FROM rustlang/rust:nightly-bullseye as builder
WORKDIR /usr/src/app
RUN apt-get update --allow-releaseinfo-change && apt-get -y install pkg-config libssl-dev
COPY src ./src/
COPY Cargo.lock Cargo.toml Makefile ./
COPY config/docker.json ./config/default.json

CMD cargo build