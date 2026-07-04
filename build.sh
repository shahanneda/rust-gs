#!/bin/sh

set -ex

wasm-pack build --release --target web
rm pkg/.gitignore
