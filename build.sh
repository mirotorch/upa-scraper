#!/bin/bash
old_home=$CARGO_HOME
export CARGO_HOME="cargo_home"
cargo build --release
export CARGO_HOME=$old_home
rm -rf cargo_home/