#!/bin/bash

cargo build --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/debug/rust-sokoban.wasm wasm
basic-http-server wasm