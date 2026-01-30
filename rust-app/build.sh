#!/bin/bash
set -e

# 1. Build the frontend WASM
echo "Building Frontend WASM..."
cd frontend
cargo build --target wasm32-unknown-unknown --release
cd ..

# 2. Generate JS bindings
echo "Generating Bindings..."
# Ensure the output directory exists
mkdir -p frontend/pkg

# Use the locally installed wasm-bindgen if globally available, or assume cargo installed it to ~/.cargo/bin
wasm-bindgen --target web --out-dir frontend/pkg --out-name frontend target/wasm32-unknown-unknown/release/frontend.wasm

# 3. Copy HTML
echo "Copying Assets..."
cp frontend/index.html frontend/pkg/index.html

echo "Build Complete."
