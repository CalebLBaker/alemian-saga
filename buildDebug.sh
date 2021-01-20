#!/bin/sh
cd game && ./buildDebug.sh && cd ../json-to-msgpack && cargo run

