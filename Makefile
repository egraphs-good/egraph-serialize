.PHONY: all test nits

all: test nits

test:
	cargo test --all-features

nits:
	@rustup component add clippy
	cargo clippy --tests --all-features -- -D warnings
	@rustup component add rustfmt
	cargo fmt --check
