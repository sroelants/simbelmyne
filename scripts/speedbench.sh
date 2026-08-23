#!/usr/bin/env bash

BLUE="\e[34m"
GREEN="\e[32m"
NORM="\e[0m"

BASE=$1
DEV=$2

RUNS=20
NODES=50000

# Check out a commit and build the binary
function build_version() {
  echo -en "Building Simbelmyne branch $BLUE$1$NORM..."
  git checkout "$1" > /dev/null 2> /dev/null

  RUSTFLAGS=-Awarnings cargo build --release -q

  echo -e "${GREEN}Done$NORM"
  cp "target/debug/simbelmyne" "/tmp/simbelmyne-$1"
  git checkout - > /dev/null 2>/dev/null
}

build_version "$BASE"
build_version "$DEV"

hyperfine \
  --runs "$RUNS" \
  --command-name "$BASE" \
  "/tmp/simbelmyne-$BASE bench -n $NODES" \
  --command-name "$DEV" \
  "/tmp/simbelmyne-$DEV bench -n $NODES"
