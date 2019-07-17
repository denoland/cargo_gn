#![deny(warnings)]

mod prebuild;
pub use prebuild::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
pub use which::which;

static GN_PATH: &'static str = env!("GN_PATH");
static NINJA_PATH: &'static str = env!("NINJA_PATH");
pub static CARGO_GN_ROOT: &'static str = env!("CARGO_GN_ROOT");

pub fn gn_path() -> PathBuf {
  PathBuf::from(GN_PATH)
}

pub fn ninja_path() -> PathBuf {
  PathBuf::from(NINJA_PATH)
}

// TODO(ry) debug_args and release_args should be Vec<String> ?
pub fn maybe_gen(root: &str, debug_args: &str, release_args: &str) -> PathBuf {
  let gn_out_dir = out_dir().join("gn_out");

  if !gn_out_dir.exists() {
    let args = if is_debug() { debug_args } else { release_args };
    write_args(&gn_out_dir.join("args.gn"), args);

    let status = Command::new(gn_path())
      .arg(format!("--root={}", root))
      .arg("gen")
      .arg(&gn_out_dir)
      .status()
      .expect("gn gen failed");
    assert!(status.success());
  }
  return gn_out_dir;
}

pub fn build(target: &str) {
  let gn_out_dir = out_dir().join("gn_out");

  // This helps Rust source files locate the snapshot, source map etc.
  println!("cargo:rustc-env=GN_OUT_DIR={}", gn_out_dir.display());

  let status = Command::new(ninja_path())
    .arg("-C")
    .arg(&gn_out_dir)
    .arg(target)
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

fn write_args(path: &PathBuf, contents: &str) {
  fs::write(path, contents).expect("Unable to write args.gn");
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
