FROM rust:1.83-slim AS builder
WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY indexer/Cargo.toml indexer/Cargo.toml
COPY . .

RUN cargo build --release --locked -p indexer

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin

COPY --from=builder /workspace/target/release/indexer .

ENV RUST_LOG=info
ENV DATABASE_URL=postgres://postgres:postgres@db:5432/ohlc
ENV ENV=dev

EXPOSE 8081
CMD ["./indexer"]
