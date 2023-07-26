.PHONY: all test nits

all: test nits

test:
	cargo test --all-features

nits:
	@rustup component add clippy
	cargo clippy --tests -- -D warnings
	@rustup component add rustfmt
	cargo fmt --check
