FROM rust:1.82-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libclang-dev clang && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build -p bach-node --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/bachledger /usr/local/bin/
ENTRYPOINT ["bachledger"]
