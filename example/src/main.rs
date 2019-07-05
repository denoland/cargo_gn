#![deny(warnings)]

#[link(name = "hello")]
extern "C" {
  fn hello();
  fn returns42() -> i32;
}

fn main() {
  assert_eq!(unsafe { returns42() }, 42);
  unsafe { hello() };
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;
  use std::process::Command;

  #[test]
  fn check_returns42() {
    assert_eq!(unsafe { returns42() }, 42);
  }

  #[test]
  fn run_foo_executable() {
    let foo = PathBuf::from(env!("OUT_DIR")).join("../../../gn_out/foo");
    assert!(foo.exists());
    let output = Command::new(foo).output().expect("foo failed");
    if cfg!(debug_assertions) {
      assert_eq!(output.stdout, b"hello debug\n");
    } else {
      assert_eq!(output.stdout, b"hello release\n");
    }
  }
}
