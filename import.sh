#!/bin/bash
# my local copy
PBCOPY='xclip -selection clipboard'
OLD=$PWD
CARGO=$HOME'/cptool'
# create mods
# $CARGO/target/debug/cptool "${@}" .. binary file are dynamic, /dep/.. /release/..
cd $CARGO
cargo run "${@}"
rustfmt .buffer.rs
$PBCOPY < .buffer.rs
cd $OLD
exec bash