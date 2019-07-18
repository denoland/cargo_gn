mod prebuild;
pub use prebuild::*;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
pub use which::which;

static GN_PATH: &'static str = env!("GN_PATH");
static NINJA_PATH: &'static str = env!("NINJA_PATH");

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
    write_args(&gn_out_dir, args);

    let status = Command::new(gn_path())
      .arg(format!("--root={}", root))
      .arg("gen")
      .arg(&gn_out_dir)
      .status()
      .expect("gn gen failed");
    assert!(status.success());
  }
  gn_out_dir
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

  rerun_if_changed(&ninja_path(), &gn_out_dir, target);

  // TODO This is not sufficent. We need to use "gn desc" to query the target
  // and figure out what else we need to add to the link.
  println!(
    "cargo:rustc-link-search=native={}/obj/",
    gn_out_dir.display()
  );
}

/// build.rs does not get re-run unless we tell cargo about what files we
/// depend on. This outputs a bunch of rerun-if-changed lines to stdout.
fn rerun_if_changed(ninja_path: &PathBuf, out_dir: &PathBuf, target: &str) {
  // TODO(ry) `ninja -t deps` isn't sufficent. It doesn't capture runtime deps.
  let deps = ninja_get_deps(ninja_path, out_dir, target);
  for d in deps {
    let p = out_dir.join(d);
    debug_assert!(p.exists());
    println!("cargo:rerun-if-changed={}", p.display());
  }
}

fn ninja_get_deps(
  ninja_path: &PathBuf,
  out_dir: &PathBuf,
  target: &str,
) -> HashSet<String> {
  let output = Command::new(ninja_path)
    .arg("-C")
    .arg(out_dir)
    .arg(target)
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

fn write_args(path: &PathBuf, contents: &str) {
  fs::create_dir_all(path).expect("Unable to create gn_out directory");
  fs::write(path.join("args.gn"), contents).expect("Unable to write args.gn");
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
