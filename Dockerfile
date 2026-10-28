# TODO: fill - verify runtime needs
FROM rust:1.80-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/tasklynx /usr/local/bin/tasklynx
ENTRYPOINT ["tasklynx"]
