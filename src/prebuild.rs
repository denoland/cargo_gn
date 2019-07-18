// This module things that are only available before cargo_gn's build.rs has
// been run.

use std::env;
use std::path::PathBuf;

pub fn out_dir() -> PathBuf {
  let out_dir = env::var("OUT_DIR").unwrap();
  let r = PathBuf::from(out_dir)
    .join("../../..")
    .canonicalize()
    .unwrap();
  r
}

pub fn is_debug() -> bool {
  // Cargo sets PROFILE to either "debug" or "release", which conveniently
  // matches the build modes we support.
  let m = env::var("PROFILE").unwrap();
  if m == "release" {
    false
  } else if m == "debug" {
    true
  } else {
    panic!("unhandled PROFILE value {}", m)
  }
}
