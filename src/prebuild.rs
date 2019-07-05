// This module things that are only available before cargo_gn's build.rs has
// been run.

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;

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

/// build.rs does not get re-run unless we tell cargo about what files we
/// depend on. This outputs a bunch of rerun-if-changed lines to stdout.
pub fn rerun_if_changed(ninja_path: &PathBuf, out_dir: &PathBuf) {
  let deps = get_deps(ninja_path, out_dir);
  for d in deps {
    // TODO(ry) Assert each file exists.
    println!("cargo:rerun-if-changed={}", d);
  }
}

pub fn get_deps(ninja_path: &PathBuf, out_dir: &PathBuf) -> HashSet<String> {
  let output = Command::new(ninja_path)
    .arg("-C")
    .arg(out_dir)
    .arg("-t")
    .arg("deps")
    .output()
    .expect("ninja failed");
  let stdout = String::from_utf8(output.stdout).unwrap();
  let mut files = HashSet::new();
  for line in stdout.lines() {
    if line.starts_with("  ") {
      files.insert(line.trim().to_string());
    }
  }
  files
}
