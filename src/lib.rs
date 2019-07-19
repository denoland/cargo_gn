use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn out_dir() -> PathBuf {
  // The OUT_DIR is going to be a crate-specific directory like
  // "target/debug/build/cargo_gn_example-eee5160084460b2c"
  // But we want to share the GN build amongst all crates
  // and return the path "target/debug". So to find it, we walk up three
  // directories.
  // TODO(ry) This is quite brittle - if Cargo changes the directory structure
  // this could break.
  let out_dir = env::var("OUT_DIR").unwrap();
  PathBuf::from(out_dir)
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .to_owned()
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

fn gn() -> String {
  env::var("GN").unwrap_or_else(|_| "gn".to_owned())
}

fn ninja() -> String {
  env::var("NINJA").unwrap_or_else(|_| "ninja".to_owned())
}

// TODO(ry) debug_args and release_args should be Vec<String> ?
pub fn maybe_gen(root: &str, debug_args: &str, release_args: &str) -> PathBuf {
  let gn_out_dir = out_dir().join("gn_out");

  if !gn_out_dir.exists() {
    let args = if is_debug() { debug_args } else { release_args };
    write_args(&gn_out_dir, args);

    println!("gn gen --root={} {}", root, gn_out_dir.display());
    let status = Command::new(gn())
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

  let status = Command::new(ninja())
    .arg("-C")
    .arg(&gn_out_dir)
    .arg(target)
    .status()
    .expect("ninja failed");
  assert!(status.success());

  rerun_if_changed(&gn_out_dir, target);

  // TODO This is not sufficent. We need to use "gn desc" to query the target
  // and figure out what else we need to add to the link.
  println!(
    "cargo:rustc-link-search=native={}/obj/",
    gn_out_dir.display()
  );
}

/// build.rs does not get re-run unless we tell cargo about what files we
/// depend on. This outputs a bunch of rerun-if-changed lines to stdout.
fn rerun_if_changed(out_dir: &PathBuf, target: &str) {
  // TODO(ry) `ninja -t deps` isn't sufficent. It doesn't capture runtime deps.
  let deps = ninja_get_deps(out_dir, target);
  for d in deps {
    let p = out_dir.join(d);
    debug_assert!(p.exists());
    println!("cargo:rerun-if-changed={}", p.display());
  }
}

fn ninja_get_deps(out_dir: &PathBuf, target: &str) -> HashSet<String> {
  let output = Command::new(ninja())
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
