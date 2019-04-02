use std::fs;
use std::process::Command;
use std::env;

mod cargo_gn {
  include!("src/prebuild.rs");
}

fn main() {
  // Build gn itself. We don't want to rely on users having it already installed
  // because it's not standard.
  let gn_mode = cargo_gn::mode();
  let out_dir = cargo_gn::out_dir();

  // TODO(ry) Use gn/build/gn.py --platform for cross compiling.
  let status = Command::new("python")
    .arg("./gn/build/gen.py")
    .arg("--no-last-commit-position")
    .arg("--out-path")
    .arg(&out_dir)
    .arg(if gn_mode == "debug" { "--debug" } else { "" })
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
  let status = Command::new("ninja")
    .arg("-C")
    .arg(&out_dir)
    .arg("gn")
    .status()
    .expect("ninja failed");
  assert!(status.success());

  cargo_gn::rerun_if_changed(&out_dir);

  let gn_path = out_dir.join("gn");
  assert!(gn_path.exists());

  println!("cargo:rustc-env=GN_PATH={}", gn_path.display());
  println!("cargo:rustc-env=CARGO_GN_ROOT={}", env::current_dir().unwrap().display());
}
