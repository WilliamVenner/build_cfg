[![crates.io](https://img.shields.io/crates/v/build_cfg.svg)](https://crates.io/crates/build_cfg)

# ✨ `build_cfg`

Test `cfg` directives at build time!

Currently, `cfg` directives do not work "correctly" in a build script. This is because the build script **must** be compiled for the host machine to execute. Therefore, it is always compiled to target the native environment.

This means that we can't conditionally compile stuff in our build scripts depending on the target platform we pass in with `--target`. This crate aims to solve this problem by collecting and evaluating `cfg` directives at runtime during the build script.

## Example

```rust
#[macro_use]
extern crate build_cfg;

#[build_cfg_main]
fn main() {
    if build_cfg!(all(target_os = "windows", target_pointer_width = "64")) {
        println!("Building for Windows 64-bit");
    } else {
        println!("Not building for Windows 64-bit");
    }
}
```