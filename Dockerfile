FROM rust:1.96-slim AS builder

WORKDIR /app

# System deps needed to compile (openssl, etc.)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# # Copy manifests first for better layer caching
# COPY Cargo.toml Cargo.lock ./
# 
# # Copy the real source + the offline query cache
# COPY src ./src
# COPY .sqlx ./.sqlx
# COPY .env ./.env

# put the fries in the bag
COPY . .

# SQLX_OFFLINE=true skips the live DB connection during compilation
ENV SQLX_OFFLINE=true

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bounty-bot ./bot
COPY .env ./.env

CMD ["./bot"]