# Stage 1: Build
FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --locked

# Stage 2: Distroless runtime
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/* /app/
EXPOSE 3000
CMD ["./app"]
