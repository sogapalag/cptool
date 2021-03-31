#!/bin/bash
# my local copy
PBCOPY='xclip -selection clipboard'
OLD=$PWD
CARGO=$HOME'/cptool'
# create mods
cd $CARGO
cargo run --bin cptool "${@}"
rustfmt .buffer.rs
$PBCOPY < .buffer.rs
cd $OLD
exec bash