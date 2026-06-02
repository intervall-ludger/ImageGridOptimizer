#!/usr/bin/env bash
# Build the WebAssembly module and JS bindings for the local web frontend.
# Requires: rustup target add wasm32-unknown-unknown, and wasm-bindgen-cli
# (brew install wasm-bindgen) matching the wasm-bindgen crate version.
set -euo pipefail

cargo build --release --target wasm32-unknown-unknown --no-default-features --features wasm
wasm-bindgen target/wasm32-unknown-unknown/release/collage_core.wasm --target web --out-dir web/pkg

echo "Built web/pkg/. Serve it with:"
echo "  python3 -m http.server --directory web 8753"
echo "then open http://localhost:8753/"
