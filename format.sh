#!/bin/sh
cd game && cargo fmt && cd ../game-lib && cargo fmt && cd ../json-to-msgpack && cargo fmt
