mod prebuild;
pub use prebuild::*;
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
}
