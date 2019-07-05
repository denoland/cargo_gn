use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod cargo_gn {
  include!("prebuild.rs");
}

fn main() {
  let out_dir = cargo_gn::out_dir();
  let ninja_path = build_ninja(&out_dir.join("_ninja_out"));
  build_gn(&out_dir.join("_gn_out"), &ninja_path);
}

fn build_ninja(out_dir: &PathBuf) -> PathBuf {
  if !out_dir.exists() {
    fs::create_dir_all(&out_dir).expect("create_dir_all");
  }

  let cargo_gn_root = env::current_dir().unwrap();
  let configure = cargo_gn_root.join("ninja/configure.py");
  let status = Command::new("python")
    .arg("-B") // PYTHONDONTWRITEBYTECODE
    .arg(configure)
    .arg("--bootstrap")
    .current_dir(&out_dir)
    .status()
    .expect("ninja/configure.py failed");
  assert!(status.success());

  let ninja_path = out_dir.join("ninja");
  println!("cargo:rustc-env=NINJA_PATH={}", ninja_path.display());
  ninja_path
}

fn build_gn(out_dir: &PathBuf, ninja: &PathBuf) {
  // TODO(ry) Use gn/build/gn.py --platform for cross compiling.
  let out_path_arg = format!("--out-path={}", out_dir.display());
  let mut gen_args = vec![
    "./gn/build/gen.py",
    "--no-last-commit-position",
    &out_path_arg,
  ];
  if cargo_gn::is_debug() {
    gen_args.push("--debug");
  }
  let status = Command::new("python")
    .args(gen_args)
    .status()
    .expect("gn/build/gen.py failed");
  assert!(status.success());

  // This is done because "cargo build" doesn't take place in a git directory,
  // which is what gn/build/gen.py needs to generate this. Hence we copy it over
  // manually.
  let last_commit_position = out_dir.join("last_commit_position.h");
  if !last_commit_position.exists() {
    fs::copy("last_commit_position.h", last_commit_position).expect("copy");
  }

  // Build gn itself.
  let status = Command::new(ninja)
    .arg("-C")
    .arg(&out_dir)
    .arg("gn")
    .status()
    .expect("ninja failed");
  assert!(status.success());

  cargo_gn::rerun_if_changed(&ninja, &out_dir);

  let gn_path = out_dir.join("gn");
  assert!(gn_path.exists());

  println!("cargo:rustc-env=GN_PATH={}", gn_path.display());
  println!(
    "cargo:rustc-env=CARGO_GN_ROOT={}",
    env::current_dir().unwrap().display()
  );
}
