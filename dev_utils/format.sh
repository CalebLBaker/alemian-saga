#!/bin/sh
cd alemian-saga && cargo fmt && cd ../alemian-saga-core && cargo fmt && cd ../dev_utils/json-to-msgpack && cargo fmt && cd ../test && cargo fmt
