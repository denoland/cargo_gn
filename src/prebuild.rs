// This module things that are only available before cargo_gn's build.rs has
// been run.

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn out_dir() -> PathBuf {
  PathBuf::from(env::var("OUT_DIR").unwrap()).join("gn_out")
}

pub fn mode() -> String {
  // Cargo sets PROFILE to either "debug" or "release", which conveniently
  // matches the build modes we support.
  let m = env::var("PROFILE").unwrap();
  String::from(if m == "release" {
    "release"
  } else if m == "debug" {
    "debug"
  } else {
    panic!("unhandled PROFILE value {}", m)
  })
}

/// build.rs does not get re-run unless we tell cargo about what files we
/// depend on. This outputs a bunch of rerun-if-changed lines to stdout.
pub fn rerun_if_changed(out_dir: &PathBuf) {
  let deps = get_deps(out_dir);
  for d in deps {
    println!("cargo:rerun-if-changed={}", d);
  }
}

pub fn get_deps(out_dir: &PathBuf) -> HashSet<String> {
  let output = Command::new("ninja")
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
