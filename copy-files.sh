#!/bin/sh
cp -r static-content/* $1
cp game/pkg/game.js $1
cp game/pkg/game_bg.wasm $1
cp generated-files/* $1

