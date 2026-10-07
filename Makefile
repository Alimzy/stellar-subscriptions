.PHONY: build test fmt lint check size
build:
	stellar contract build --package subscriptions
test:
	cargo test --workspace --locked
fmt:
	cargo fmt --all
lint:
	cargo clippy --workspace --all-targets --locked -- -D warnings
check: fmt lint test build size
size:
	@ls -l target/wasm32v1-none/release/subscriptions.wasm
