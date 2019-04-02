mod prebuild;
pub use prebuild::*;
use std::env;
use std::path::PathBuf;
use std::process::Command;

static GN_PATH: &'static str = env!("GN_PATH");
static CARGO_GN_ROOT: &'static str = env!("CARGO_GN_ROOT");

pub fn gn_path() -> PathBuf {
  PathBuf::from(GN_PATH)
}

pub fn build(root: &str, target: &str, release_args: &str, debug_args: &str) {
  println!("hello from cargo_gn::build");

  // let default_dotfile = PathBuf::from(CARGO_GN_ROOT).join("default.gn");
  // assert!(default_dotfile.exists());

  let gn_out_dir = out_dir();
  println!("gn_out_dir {}", gn_out_dir.display());

  let gn = gn_path();
  assert!(gn.exists());

  //println!("current_dir {}", env::current_dir().unwrap().display());

  let status = Command::new(&gn)
    //.arg(format!("--root={}", root))
    //.arg(format!("--dotfile={}", default_dotfile.display()))
    .arg("gen")
    .arg(&gn_out_dir)
    .status()
    .expect("gn gen failed");
  assert!(status.success());


  let status = Command::new("ninja")
      .arg("-C")
      .arg(&gn_out_dir)
      .arg(target)
      .status()
      .expect("ninja failed");
  assert!(status.success());
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
