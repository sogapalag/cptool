#!/bin/bash
# my local copy
PBCOPY='xclip -selection clipboard'
CARGO=$HOME'/cptool'
# create mods
$CARGO/target/debug/cptool "${@}"
rustfmt $CARGO/.buffer.rs
$PBCOPY < $CARGO/.buffer.rs