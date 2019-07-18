#!/bin/bash

if command -v sccache; then
  export RUSTC_WRAPPER="sccache"
  export CXX="sccache clang++"
fi

cargo test -vv --all
