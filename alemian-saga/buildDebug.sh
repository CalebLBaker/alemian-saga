#!/bin/sh
wasm-pack build --target no-modules --no-typescript --dev -- --features "stack-trace"

