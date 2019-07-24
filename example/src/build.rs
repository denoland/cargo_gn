use cargo_gn;

fn main() {
  let gn_args = if cargo_gn::is_debug() {
    vec![("is_debug".to_string(), "true".to_string())]
  } else {
    vec![("is_debug".to_string(), "false".to_string())]
  };
  let gn_out = cargo_gn::maybe_gen(".", gn_args);
  assert!(gn_out.exists());
  assert!(gn_out.join("args.gn").exists());
  cargo_gn::build("default");
}
