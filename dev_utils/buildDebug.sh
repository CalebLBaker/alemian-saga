#!/bin/sh
set -e
cd alemian-saga
./buildDebug.sh
cd ../dev_utils/json-to-msgpack
cargo run
if [[ $# -eq 0 ]] ; then
    exit 0
fi
cd ../..
cp -r public/* $1
cp alemian-saga/pkg/alemian_saga.js $1
cp alemian-saga/pkg/alemian_saga_bg.wasm $1
cp -r dev_utils/generated-files/* $1

