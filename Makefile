.PHONY: default help setup clean fmt build test lint security-scan docs component cli web e2e dev ayce
default: ayce

# Playwright needs system libraries for Chromium on Linux (it asks for sudo);
# macOS has them.
PLAYWRIGHT_DEPS := $(if $(filter Linux,$(shell uname -s)),--with-deps,)

help:  ## self-documenting: every target has a '## comment' printed here
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-16s %s\n", $$1, $$2}'

setup: ## one-time setup for make ayce: wasm32-wasip2, cargo-deny, cargo-audit, npm packages, Chromium (needs rustup, node)
	@command -v rustup >/dev/null || { echo "setup: install rustup first: https://rustup.rs"; exit 1; }
	@command -v npm >/dev/null || { echo "setup: install Node first (version in .tool-versions)"; exit 1; }
	rustup target add wasm32-wasip2
	cargo install cargo-deny cargo-audit
	cd component/js && npm ci
	cd web && npm ci && npx playwright install $(PLAYWRIGHT_DEPS) chromium

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

component: ## build the WIT component, run the golden fixtures against it (needs make setup)
	cargo build -p buffoonery-component --target wasm32-wasip2 --profile component
	cd component/js && npm ci && npm test

cli: ## build the buffoonery command-line tool into target/debug/buffoonery
	cargo build -p buffoonery-cli

web: component cli ## test and build the Joker Calculator page into web/dist (needs make setup)
	cd web && npm ci && npm test && npx vite build

e2e: web ## browser parity test: every golden fixture in the real page (needs make setup)
	cd web && npx playwright test

dev: ## serve the Joker Calculator with live reload; rerun after a Rust change (needs make setup)
	cargo build -p buffoonery-component --target wasm32-wasip2 --profile component
	cd component/js && { [ -d node_modules ] || npm ci; } && npm run opt
	cd web && { [ -d node_modules ] || npm ci; } && npm run dev

ayce: clean fmt build test lint security-scan docs e2e ## all-you-can-eat: full pre-push sweep

.PHONY: watch install-tools

watch: ## rebuild/retest on save (requires cargo-watch)
	cargo watch -x "check --workspace" -x "test --workspace"

install-tools: ## install cargo-deny, cargo-audit, and cargo-watch (make setup installs the first two)
	cargo install cargo-deny
	cargo install cargo-audit
	cargo install cargo-watch
