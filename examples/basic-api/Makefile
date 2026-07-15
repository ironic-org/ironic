.PHONY: build test dev fmt clippy docker-build docker-up docker-down clean

build:
	cargo build

test:
	cargo test -- --test-threads=1

dev:
	cargo run -- dev

fmt:
	cargo fmt --all

clippy:
	cargo clippy -- -D warnings

docker-build:
	docker build -t app .

docker-up:
	docker compose up -d

docker-down:
	docker compose down

clean:
	cargo clean
