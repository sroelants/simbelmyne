#!/usr/bin/env bash

BLUE="\e[34m"
GREEN="\e[32m"
NORM="\e[0m"

BASE=$1
DEV=$2

# Check out a commit and build the binary
function build_version() {
  echo -en "Building Simbelmyne branch $BLUE$1$NORM..."
  git checkout "$1" > /dev/null

  RUSTFLAGS=-Awarnings cargo build -q

  echo -e "${GREEN}Done$NORM"
  cp "target/debug/simbelmyne" "/tmp/simbelmyne-$1"
  git checkout - > /dev/null 2>/dev/null
}

build_version "$BASE"
build_version "$DEV"

RUST_BACKTRACE=1 fast-chess \
  -engine proto=uci cmd="/tmp/simbelmyne-$BASE" name="$BASE" \
  -engine proto=uci cmd="/tmp/simbelmyne-$DEV" name="$DEV" \
  -each tc=8+0.08 \
  -games 2 \
  -rounds 50000 \
  -repeat \
  -concurrency 12 \
  -ratinginterval 10 \
  -openings file="/home/sam/bin/4moves_noob.epd" format=epd order=random \
  -randomseed \
  -sprt elo0=0 elo1=5 alpha=0.05 beta=0.1 \
  -recover \
  -log file="/tmp/sprt-$BASE-$DEV-output.log"
