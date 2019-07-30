#!/bin/bash
# Be sure to set $NINJA and $GN env vars correctly before running this script.
CARGO_TEST="cargo test -vv --all"

# First we run cargo test, to make sure everything is built.
$CARGO_TEST || exit 39

# If we run cargo test again, we should not rebuild.
$CARGO_TEST | grep count_bytes && exit 40
$CARGO_TEST | grep CXX && exit 41

# Rebuild if we touch an explicitly listed source file
touch example/src/hello.cc
$CARGO_TEST | grep CXX || exit 42

# Rebuild If we touch the input to an action
touch example/src/input.txt
$CARGO_TEST | grep count_bytes || exit 43

# Rebuild if we touch a header file not directly listed in the BUILD.gn
touch example/src/hello.h
$CARGO_TEST | grep CXX || exit 44

# TODO(ry) Rebuild if we touch a BUILD.gn file.
# touch example/BUILD.gn
# $CARGO_TEST | grep "gen ." || exit 45
