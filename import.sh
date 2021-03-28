#!/bin/bash
# my local copy
PBCOPY='xclip -selection clipboard'
OLD=$PWD
CARGO=$HOME'/cptool'
# create mods
cd $CARGO
cargo run "${@}"
rustfmt .buffer.rs
$PBCOPY < .buffer.rs
cd $OLD
exec bash