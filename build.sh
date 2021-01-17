#!/bin/sh
cd game && ./buildRelease.sh && cd ../json-to-msgpack && cargo run

