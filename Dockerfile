FROM rust:latest AS builder


WORKDIR /usr/src/vhennus_server
RUN apt-get update && apt-get install -y \
    ca-certificates \
    # The 'libssl3' package (which provides libssl.so.3) is usually installed by default
    # in bookworm-slim, but you can explicitly install it if needed:
    # libssl3 \
    && rm -rf /var/lib/apt/lists/*


COPY log4rs.yaml .env Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo build --release --locked

FROM debian:bookworm-slim

WORKDIR /usr/local/bin

RUN apt-get update && apt-get install -y \
    ca-certificates \
    # The 'libssl3' package (which provides libssl.so.3) is usually installed by default
    # in bookworm-slim, but you can explicitly install it if needed:
    # libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y libssl3

COPY --from=builder /usr/src/vhennus_server/target/release/vhennus_server ./
COPY log4rs.yaml ./
COPY .env ./

RUN chmod +x vhennus_server
CMD ["./vhennus_server"]