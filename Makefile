RUST_APP = rust-app
CARGO = cargo
NPM = npm

.PHONY: all build test test-e2e clean

all: build

# Delegate to the Rust workspace (Leptos WASM frontend + Axum backend).
build:
	cd $(RUST_APP) && bash build.sh

test:
	cd $(RUST_APP) && $(CARGO) test --workspace

test-e2e: build
	cd $(RUST_APP)/tests/e2e && $(NPM) ci && npx playwright test

clean:
	cd $(RUST_APP) && $(CARGO) clean
