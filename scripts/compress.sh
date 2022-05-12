#!/bin/bash

wasm-snip --snip-rust-panicking-code --snip-rust-fmt-code pkg/automaton_engine_bg.wasm -o target/tmp/her.wasm
wasm-opt --dce target/tmp/her.wasm -o target/tmp/her2.wasm
wasm-opt -Oz target/tmp/her2.wasm -o target/tmp/her3.wasm
mv target/tmp/her3.wasm pkg/automaton_engine_bg.wasm
rm -rf target/tmp/her.wasm target/tmp/her2.wasm
