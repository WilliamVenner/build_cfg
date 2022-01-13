#![doc = include_str!("../../README.md")]

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