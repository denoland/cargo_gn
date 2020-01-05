mod deps;

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

#[derive(Clone, Debug)]
struct Dirs {
  pub out: PathBuf,
  pub root: PathBuf,
}

fn get_dirs(manifest_dir: Option<&str>) -> Dirs {
  // The OUT_DIR is going to be a crate-specific directory like
  // "target/debug/build/cargo_gn_example-eee5160084460b2c"
  // But we want to share the GN build amongst all crates
  // and return the path "target/debug". So to find it, we walk up three
  // directories.
  // TODO(ry) This is quite brittle - if Cargo changes the directory structure
  // this could break.
  let out = env::var("OUT_DIR").map(PathBuf::from).unwrap();
  let out = out
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .to_owned();

  let root = match manifest_dir {
    Some(s) => env::current_dir().unwrap().join(s),
    None => env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap(),
  };

  let mut dirs = Dirs { out, root };
  maybe_symlink_root_dir(&mut dirs);
  dirs
}

#[cfg(not(target_os = "windows"))]
fn maybe_symlink_root_dir(_: &mut Dirs) {}

#[cfg(target_os = "windows")]
fn maybe_symlink_root_dir(dirs: &mut Dirs) {
  // GN produces invalid paths if the source (a.k.a. root) directory is on a
  // different drive than the output. If this is the case we'll create a
  // symlink called "gn_root' in the out directory, next to 'gn_out', so it
  // appears as if they're both on the same drive.
  use std::fs::remove_dir;
  use std::os::windows::fs::symlink_dir;
  use std::path::{Component, Path};

  let get_prefix = |p: &Path| {
    p.components()
      .find_map(|c| match c {
        Component::Prefix(p) => Some(p),
        _ => None,
      })
      .map(|p| p.as_os_str().to_owned())
  };

  let Dirs { out, root } = dirs;
  if get_prefix(out) != get_prefix(root) {
    let symlink = &*out.join("gn_root");
    let target = &*root.canonicalize().unwrap();

    println!("Creating symlink {:?} to {:?}", &symlink, &root);

    loop {
      match symlink.canonicalize() {
        Ok(existing) if existing == target => break,
        Ok(_) => remove_dir(symlink).expect("remove_dir failed"),
        Err(_) => {
          break symlink_dir(target, symlink).expect("symlink_dir failed")
        }
      }
    }

    dirs.root = symlink.to_path_buf();
  }
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

pub type NinjaEnv = Vec<(String, String)>;

fn ninja(gn_out_dir: &PathBuf, maybe_env: Option<NinjaEnv>) -> Command {
  let cmd_string = env::var("NINJA").unwrap_or_else(|_| "ninja".to_owned());
  let mut cmd = Command::new(cmd_string);
  cmd.arg("-C");
  cmd.arg(&gn_out_dir);
  if let Some(env) = maybe_env {
    for item in env {
      cmd.env(item.0, item.1);
    }
  }
  cmd
}

pub type GnArgs = Vec<String>;

pub fn maybe_gen(manifest_dir: &str, gn_args: GnArgs) -> PathBuf {
  let dirs = get_dirs(Some(manifest_dir));
  let gn_out_dir = dirs.out.join("gn_out");

  if !gn_out_dir.exists() || !gn_out_dir.join("build.ninja").exists() {
    let args = gn_args.join(" ");

    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());
    println!(
      "gn gen --root={} {}",
      dirs.root.display(),
      gn_out_dir.display()
    );
    let mut cmd = Command::new(gn());
    cmd.arg(format!("--root={}", dirs.root.display()));
    cmd.arg("gen");
    cmd.arg(&gn_out_dir);
    cmd.arg("--args=".to_owned() + &args);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.envs(env::vars());
    run(&mut cmd, "gn gen");
  }
  gn_out_dir
}

pub fn build(target: &str, maybe_env: Option<NinjaEnv>) {
  let gn_out_dir = get_dirs(None).out.join("gn_out");

  // This helps Rust source files locate the snapshot, source map etc.
  println!("cargo:rustc-env=GN_OUT_DIR={}", gn_out_dir.display());

  let mut cmd = ninja(&gn_out_dir, maybe_env.clone());
  cmd.arg(target);
  run(&mut cmd, "ninja");

  rerun_if_changed(&gn_out_dir, maybe_env, target);

  // TODO This is not sufficent. We need to use "gn desc" to query the target
  // and figure out what else we need to add to the link.
  println!(
    "cargo:rustc-link-search=native={}/obj/",
    gn_out_dir.display()
  );
}

/// build.rs does not get re-run unless we tell cargo about what files we
/// depend on. This outputs a bunch of rerun-if-changed lines to stdout.
fn rerun_if_changed(
  out_dir: &PathBuf,
  maybe_env: Option<NinjaEnv>,
  target: &str,
) {
  let deps = deps::ninja_get_deps(out_dir, maybe_env, target);
  for d in deps {
    let p = out_dir.join(d);
    assert!(p.exists());
    println!("cargo:rerun-if-changed={}", p.display());
  }
}

fn run(cmd: &mut Command, program: &str) {
  use std::io::ErrorKind;
  println!("running: {:?}", cmd);
  let status = match cmd.status() {
    Ok(status) => status,
    Err(ref e) if e.kind() == ErrorKind::NotFound => {
      fail(&format!(
        "failed to execute command: {}\nis `{}` not installed?",
        e, program
      ));
    }
    Err(e) => fail(&format!("failed to execute command: {}", e)),
  };
  if !status.success() {
    fail(&format!(
      "command did not execute successfully, got: {}",
      status
    ));
  }
}

fn fail(s: &str) -> ! {
  panic!("\n{}\n\nbuild script failed, must exit now", s)
}
