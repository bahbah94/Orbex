FROM rust:1.90-slim AS builder
WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY tradebot/Cargo.toml tradebot/Cargo.toml
COPY . .

RUN cargo build --release --locked -p tradebot

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin

COPY --from=builder /workspace/target/release/tradebot .

ENV RUST_LOG=debug
ENV ENV=dev
ENV NUM_ACCOUNTS=6
ENV ORDER_DATA_FILE=data.jsonl

COPY ${ORDER_DATA_FILE} ./

CMD ["./orbex-tradebot"]