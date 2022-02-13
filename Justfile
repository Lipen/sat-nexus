# just manual: https://github.com/casey/just/#readme

_default:
	@just --list --unsorted

# Run clippy
check:
	cargo clippy --workspace --all-targets

# Run all tests
test:
	cargo test --workspace
