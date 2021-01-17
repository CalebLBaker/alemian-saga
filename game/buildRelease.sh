#!/bin/sh
wasm-pack build --target no-modules --no-typescript --release -- --features "strict" \
&& wasm-gc pkg/game_bg.wasm \
&& wasm-opt -O3 -o pkg/game_bg.wasm pkg/game_bg.wasm

