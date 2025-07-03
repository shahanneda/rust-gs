#!/bin/sh

set -ex

wasm-pack build --dev --target web
rm pkg/.gitignore