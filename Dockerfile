# Build stage
FROM rust:bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Final run stage
FROM debian:bookworm-slim AS runner

WORKDIR /app
COPY --from=builder /app/target/release/podcast_crawler /app/podcast_crawler
RUN apt-get update && apt-get install -y libssl3
CMD ["/app/podcast_crawler"]
