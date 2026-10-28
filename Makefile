.PHONY: build release test clippy fmt run

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

run:
	cargo run -- $(ARGS)
