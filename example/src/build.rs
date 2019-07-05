use cargo_gn;

fn main() {
  let gn_out = cargo_gn::maybe_gen(".", "is_debug=true", "is_debug=false");
  assert!(gn_out.exists());
  assert!(gn_out.join("args.gn").exists());
  cargo_gn::build("default");
}
