#!/bin/sh

set -ex

cargo build --bin local
cargo run --bin local
