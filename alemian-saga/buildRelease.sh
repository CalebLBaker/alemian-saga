#!/bin/sh
wasm-pack build --target no-modules --no-typescript --release -- --features "strict" \
&& wasm-gc pkg/alemian_saga_bg.wasm \
&& wasm-opt -O3 -o pkg/alemian_saga_bg.wasm pkg/alemian_saga_bg.wasm

