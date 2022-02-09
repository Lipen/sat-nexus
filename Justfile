# just manual: https://github.com/casey/just/#readme

_default:
	@just --list --unsorted

# Run clippy
check:
	cargo clippy -- -D warnings

# Run all tests
test:
	cargo test --all
