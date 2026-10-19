.PHONY: default help clean fmt build test lint security-scan docs component ayce
default: ayce

help:  ## self-documenting: every target has a '## comment' printed here
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-16s %s\n", $$1, $$2}'

clean: ## remove build artifacts
	cargo clean

fmt: ## format all sources
	cargo fmt

build: ## compile/build
	cargo build --workspace

test: ## run all tests
	cargo test --workspace

lint: ## static analysis at the pedantic end
	cargo clippy --workspace --all-targets -- -D warnings
	cargo clippy -p buffoonery-component --target wasm32-wasip2 -- -D warnings
	cargo deny check

security-scan: ## dependency vulnerability scan
	./bin/security-scan

docs: ## build docs, fail on warnings
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

component: ## build the WIT component, run the golden fixtures against it (needs wasm32-wasip2, node)
	cargo build -p buffoonery-component --target wasm32-wasip2 --profile component
	cd component/js && npm ci && npm test

ayce: clean fmt build test lint security-scan docs component ## all-you-can-eat: full pre-push sweep

.PHONY: watch install-tools

watch: ## rebuild/retest on save (requires cargo-watch)
	cargo watch -x "check --workspace" -x "test --workspace"

install-tools: ## install cargo-deny, cargo-audit, and cargo-watch
	cargo install cargo-deny
	cargo install cargo-audit
	cargo install cargo-watch
