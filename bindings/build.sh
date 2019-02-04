#!/bin/sh

cargo build --release --target=wasm32-unknown-unknown
mkdir lib 2>/dev/null || true
cp target/wasm32-unknown-unknown/release/bindings.wasm lib
wasm-bindgen --nodejs --out-dir lib lib/bindings.wasm
yarn
yarn tsc
