use std::env;
use std::fs;
use std::path::{self, Path};
use std::process::Command;

fn main() {
    // Build gn itself. We don't want to rely on users having it already installed
    // because it's not standard.

    // Cargo sets PROFILE to either "debug" or "release", which conveniently
    // matches the build modes we support.
    let gn_mode = env::var("PROFILE").unwrap();

    let root = env::current_dir().unwrap();
    let gn_out_path = root.join(format!("target/{}/gn_out", gn_mode.clone()));
    let gn_out_dir = normalize_path(&gn_out_path);

    // TODO(ry) Use gn/build/gn.py --platform for cross compiling.
    let status = Command::new("python")
        .arg("./gn/build/gen.py")
        .arg("--no-last-commit-position")
        .arg("--out-path")
        .arg(&gn_out_dir)
        .arg(if gn_mode == "debug" { "--debug" } else { "" })
        .status()
        .expect("gn/build/gen.py failed");
    assert!(status.success());

    // This is done because "cargo build" doesn't take place in a git directory,
    // which is what gn/build/gen.py needs to generate this. Hence we copy it over
    // manually.
    fs::copy(
        "last_commit_position.h",
        gn_out_path.join("last_commit_position.h"),
    )
    .expect("copy");

    // Build gn itself.
    let status = Command::new("ninja")
        .env("CFLAGS", format!("-I{}", root.display()))
        .arg("-C")
        .arg(&gn_out_dir)
        .arg("gn")
        .status()
        .expect("ninja failed");
    assert!(status.success());
}

/// Utility function to make a path absolute, normalizing it to use forward
/// slashes only. The returned value is an owned String, otherwise panics.
fn normalize_path<T: AsRef<Path>>(path: T) -> String {
    path.as_ref()
        .to_str()
        .unwrap()
        .to_owned()
        .chars()
        .map(|c| if path::is_separator(c) { '/' } else { c })
        .collect()
}
