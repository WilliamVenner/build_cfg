//! [![crates.io](https://img.shields.io/crates/v/build_cfg.svg)](https://crates.io/crates/build_cfg)
//!
//! # ✨ `build_cfg`
//!
//! Test `cfg` directives at build time!
//!
//! Currently, `cfg` directives do not work "correctly" in a build script. This is because the build script **must** be compiled for the host machine to execute. Therefore, it is always compiled to target the native environment.
//!
//! This means that we can't conditionally compile stuff in our build scripts depending on the target platform we pass in with `--target`. This crate aims to solve this problem by collecting and evaluating `cfg` directives at runtime during the build script.
//!
//! ## Example
//!
//! ```rust
//! #[macro_use]
//! extern crate build_cfg;
//!
//! #[build_cfg_main]
//! fn main() {
//!     if build_cfg!(all(target_os = "windows", target_pointer_width = "64")) {
//!         println!("Building for Windows 64-bit");
//!     } else {
//!         println!("Not building for Windows 64-bit");
//!     }
//! }
//! ```

pub use build_cfg_macros::*;

#[doc(hidden)]
pub mod __private {
	pub use build_cfg_shared::*;

	use std::ffi::OsString;

	pub fn populate_cfg(vars_os: impl Iterator<Item = (OsString, OsString)>) {
		let mut cfg = EnvCfg::default();

		let mut found = false;
		for (key, val) in vars_os {
			let (key, val) = (key.to_string_lossy(), val.to_string_lossy());
			if let Some(key) = key.strip_prefix("CARGO_CFG_") {
				let key = key.to_ascii_lowercase();
				if let Some(key) = CfgKey::from_str(key) {
					key.lookup_mut(&mut cfg).put(val.into_owned());
					found = true;
				}
			} else if let Some(feature) = key.strip_prefix("CARGO_FEATURE_") {
				cfg.feature.insert(feature.to_ascii_lowercase());
				found = true;
			}
		}
		if !found {
			panic!("Not in a Cargo/build script environment.");
		}

		__TARGET_CFG.with(|target_cfg| *target_cfg.borrow_mut() = Some(cfg));
	}
}