FROM rust:1.90-slim AS builder
WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY indexer/Cargo.toml indexer/Cargo.toml
COPY . .

RUN cargo build --release --locked -p indexer

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin

COPY --from=builder /workspace/target/release/indexer .

ENV INDEXER_PORT=8081
EXPOSE ${INDEXER_PORT}

CMD ["./orbex-indexer"]
