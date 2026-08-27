#!/usr/bin/env bash

RED="\e[31m"
GREEN="\e[32m"
BLUE="\e[34m"
NORM="\e[0m"

BASE=$1
DEV=$2

RUNS=100
NODES=10000


# Check out a commit and build the binary
function build_version() {
  git checkout "$1" > /dev/null 2> /dev/null

  if [ ! $? -eq 0 ]; then
    echo -e "${RED}[ERR]:${NORM} Failed to check out branch $BLUE$1$NORM."
    echo -en "Output: "
    git checkout "$1" > /dev/null
    exit 1
  fi

  DEST="/tmp/simbelmyne-$(git rev-parse --short HEAD)"

  if [ ! -e "$DEST" ]; then
    echo -en "Building Simbelmyne branch $BLUE$1$NORM..." >&2
    RUSTFLAGS="-Awarnings -Ctarget-cpu=native" cargo build --release -q
    echo -e "${GREEN}Done$NORM" >&2
    cp "target/release/simbelmyne" "$DEST"
  else
    echo -e "Found cached file for Simbelmyne $BLUE$1$NORM, skipping build." >&2
  fi

  git checkout - > /dev/null 2>/dev/null

  # Return the binary path
  echo "$DEST"
}

BASE_PATH=$(build_version "$BASE")
DEV_PATH=$(build_version "$DEV")

hyperfine \
  --runs "$RUNS" \
  --command-name "$BASE" \
  "$BASE_PATH bench -n $NODES" \
  --command-name "$DEV" \
  "$DEV_PATH bench -n $NODES"

