# Build stage
FROM rust:1.80-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY assets ./assets
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN useradd -m -u 1000 tasklynx
COPY --from=builder /app/target/release/tasklynx /usr/local/bin/tasklynx
USER tasklynx
ENTRYPOINT ["tasklynx"]
CMD ["--help"]
