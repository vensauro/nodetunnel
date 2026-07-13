FROM rust:1.88-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/protocol/Cargo.toml crates/protocol/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml
COPY crates/client/Cargo.toml crates/client/Cargo.toml

RUN mkdir -p crates/protocol/src crates/server/src crates/client/src && \
    echo "" > crates/protocol/src/lib.rs && \
    echo "fn main() {}" > crates/server/src/main.rs && \
    echo "" > crates/client/src/lib.rs

RUN cargo fetch

COPY crates/ crates/

RUN cargo build --release -p nt_server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/nt_server /usr/local/bin/app
WORKDIR /
EXPOSE 8080/udp
ENTRYPOINT ["/usr/local/bin/app"]
