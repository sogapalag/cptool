#!/bin/bash
# my local copy
PBCOPY='xclip -selection clipboard'
# create mods
~/cptool/target/debug/cptool "${@}"
rustfmt ~/cptool/.buffer.rs
$PBCOPY < ~/cptool/.buffer.rs