# Cargo GN integration

[![Build Status](https://dev.azure.com/denoland/cargo_gn/_apis/build/status/denoland.cargo_gn?branchName=master)](https://dev.azure.com/denoland/cargo_gn/_build/latest?definitionId=3&branchName=master)

This package allows Rust users to quickly hook into the GN build system.

Put the following in your Cargo.toml

```toml
[build-dependencies]
cargo_gn = "0.0.2"
```

And put this exact code in your build.rs

```rust
use cargo_gn;

fn main() {
  cargo_gn::main()
}
```

Now you should be able to put a `.gn` file in the root of your project and
create `BUILD.gn` files. See the example directory for a complete example:
https://github.com/denoland/cargo_gn/tree/master/example

Use `cargo build -vv` in order to see ninja output.
