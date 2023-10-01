#!/bin/bash
set -eu
cargo b --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./dist/ --out-name wasm --target web --no-typescript ./target/wasm32-unknown-unknown/release/ld54.wasm
cp -r ./assets dist/
cp ./wasm/index.html dist/
zip -r dist.zip ./dist
