build:
	cargo build

test: build
	cargo test -- --nocapture

lint: format clippy check

format:
	cargo fmt --all -- --check

fix:
	cargo fix

clippy:
	cargo clippy -- -D warnings

check:
	cargo check -v

.PHONY: build test format fix clippy check lint

