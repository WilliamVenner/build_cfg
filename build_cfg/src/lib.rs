pub use build_cfg_macros::*;

/// # Do not touch!
pub mod __private {
	pub use build_cfg_shared::*;

	use std::ffi::OsString;

	pub fn populate_cfg(vars_os: impl Iterator<Item = (OsString, OsString)>) {
		let mut found = false;
		let mut cfg = EnvCfg::default();

		vars_os
			.filter_map(|(key, val)| {
				if val.is_empty() {
					None
				} else {
					key.to_string_lossy()
						.strip_prefix("CARGO_CFG_")
						.map(|key| key.to_ascii_lowercase())
						.and_then(|key| CfgKey::from_str(key))
						.map(|key| {
							found = true;
							(key, val.to_string_lossy().into_owned())
						})
				}
			})
			.for_each(|(key, val)| {
				key.lookup_mut(&mut cfg).put(val);
			});

		if !found {
			panic!("Not in a Cargo/build script environment.");
		}

		__TARGET_CFG.with(|target_cfg| *target_cfg.borrow_mut() = Some(cfg));
	}
}