# Stage 1: Build
FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# Stage 2: Distroless runtime
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/todo_example /app/todo_example
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
EXPOSE 3000
CMD ["./todo_example"]
