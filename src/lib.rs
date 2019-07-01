mod prebuild;
pub use prebuild::*;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

static GN_PATH: &'static str = env!("GN_PATH");
static NINJA_PATH: &'static str = env!("NINJA_PATH");
pub static CARGO_GN_ROOT: &'static str = env!("CARGO_GN_ROOT");

pub fn gn_path() -> PathBuf {
  PathBuf::from(GN_PATH)
}

pub fn ninja_path() -> PathBuf {
  PathBuf::from(NINJA_PATH)
}

pub struct Config {
  root: String,
  target: String,
  release_args: String,
  debug_args: String,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      root: ".".to_string(),
      target: "default".to_string(),
      release_args: "is_debug=false".to_string(),
      debug_args: "is_debug=true".to_string(),
    }
  }
}

pub fn main() {
  build(&Config::default())
}

fn write_args(path: &PathBuf, contents: &str) {
  fs::write(path, contents).expect("Unable to write args.gn");
}

pub fn build(c: &Config) {
  let gn_out_dir = out_dir().join("gn_out");

  let deps = ninja_deps(&gn_out_dir);
  for dep in deps.iter() {
    println!("cargo:rerun-if-changed={}", dep);
  }

  let status = Command::new(gn_path())
    .arg(format!("--root={}", c.root))
    .arg("gen")
    .arg(&gn_out_dir)
    .status()
    .expect("gn gen failed");
  assert!(status.success());

  let args = if is_debug() {
    &c.debug_args
  } else {
    &c.release_args
  };
  write_args(&gn_out_dir.join("args.gn"), args);

  let status = Command::new(ninja_path())
    .arg("-C")
    .arg(&gn_out_dir)
    .arg(&c.target)
    .status()
    .expect("ninja failed");
  assert!(status.success());

  // TODO This is not sufficent. We need to use "gn desc" to query the target
  // and figure out what else we need to add to the link.
  println!(
    "cargo:rustc-link-search=native={}/obj/",
    gn_out_dir.display()
  );
}

pub fn ninja_deps(gn_out_dir: &PathBuf) -> HashSet<String> {
  let output = Command::new(ninja_path())
    .arg("-C")
    .arg(gn_out_dir)
    .arg("-t")
    .arg("deps")
    .output()
    .expect("ninja -t deps failed");

  let stdout = std::str::from_utf8(&output.stdout).unwrap();
  ninja_deps_parse(&stdout)
}

pub fn ninja_deps_parse(s: &str) -> HashSet<String> {
  let mut out = HashSet::new();
  for line in s.lines() {
    if line.starts_with("    ") {
      let filename = String::from(line.trim_start());
      out.insert(filename);
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn gn_exists() {
    assert!(gn_path().exists());
  }

  #[test]
  fn ninja_exists() {
    assert!(ninja_path().exists());
  }

  const NINJA_DEPS_OUTPUT: &'static str = r#"
obj/foo/foo.o: #deps 2, deps mtime 1562002734 (VALID)
    ../../../../../../example/src/foo.cc
    ../../../../../../example/src/hello.h

obj/hello/hello.o: #deps 2, deps mtime 1562002734 (VALID)
    ../../../../../../example/src/hello.cc
    ../../../../../../example/src/hello.h
  "#;

  #[test]
  fn test_parse_ninja_deps_output() {
    let foo_cc = "../../../../../../example/src/foo.cc".to_string();
    let hello_cc = "../../../../../../example/src/hello.cc".to_string();
    let hello_h = "../../../../../../example/src/hello.h".to_string();
    let blah_h = "../../../../../../example/src/blah.h".to_string();

    let set = ninja_deps_parse(NINJA_DEPS_OUTPUT);

    assert!(set.contains(&foo_cc));
    assert!(set.contains(&hello_cc));
    assert!(set.contains(&hello_h));
    assert!(!set.contains(&blah_h));
  }
}
