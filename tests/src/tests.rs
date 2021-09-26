macro_rules! setup_test {
	($($key:literal: $val:literal),*) => {
		$(std::env::set_var($key, $val);)*
		build_cfg::__private::populate_cfg(std::env::vars_os().filter(|(key, _)| match key.to_string_lossy().as_ref() {
			$($key => true,)*
			_ => false
		}).into_iter());
	};
}

fn clear_state() {
	build_cfg::__private::__TARGET_CFG.with(|target_cfg| target_cfg.borrow_mut().take());
}

#[test]
#[should_panic]
fn test_state_uninit() {
	clear_state();
	build_cfg::__private::env_cfg(|_| {});
}

#[test]
#[should_panic]
fn test_state_empty() {
	clear_state();
	build_cfg::__private::populate_cfg(std::env::vars_os().filter(|(key, _)| !key.to_string_lossy().starts_with("CARGO_CFG_")).into_iter());
}

#[test]
fn test_state() {
	clear_state();
	setup_test!("CARGO_CFG_TARGET_OS": "windows");
	build_cfg::__private::populate_cfg(std::env::vars_os());
}

fn __test_basic() {
	assert!(build_cfg!(all(target_os = "windows", target_pointer_width = "64")));
	assert!(!build_cfg!(not(all(target_os = "windows", target_pointer_width = "64"))));
	assert!(!build_cfg!(all(target_os = "windows", target_pointer_width = "32")));
	assert!(build_cfg!(any(target_os = "windows", target_pointer_width = "32")));
	assert!(build_cfg!(any(target_os = "windows", target_pointer_width = "64")));
	assert!(build_cfg!(any(target_os = "linux", target_pointer_width = "64")));
	assert!(!build_cfg!(all(target_os = "linux", target_pointer_width = "64")));
	assert!(build_cfg!(target_os = "windows"));
	assert!(!build_cfg!(target_os = "linux"));
}
#[test]
fn test_basic() {
	setup_test!(
		"CARGO_CFG_TARGET_OS": "windows",
		"CARGO_CFG_TARGET_POINTER_WIDTH": "64"
	);
	__test_basic();
}

fn __test_set() {
	assert!(build_cfg!(target_feature = "mmx"));
	assert!(build_cfg!(target_feature = "sse"));
	assert!(!build_cfg!(not(target_feature = "mmx")));
	assert!(!build_cfg!(not(target_feature = "sse")));
	assert!(build_cfg!(any(target_feature = "mmx", target_feature = "sse")));
	assert!(!build_cfg!(not(any(target_feature = "mmx", target_feature = "sse"))));
	assert!(build_cfg!(all(target_feature = "mmx", target_feature = "sse")));
	assert!(!build_cfg!(all(target_feature = "mmx", target_feature = "sse", target_feature = "lol")));
	assert!(!build_cfg!(target_feature = "lol"));
}
#[test]
fn test_set() {
	setup_test!(
		"CARGO_CFG_TARGET_FEATURE": "mmx,sse"
	);
	__test_set();
}

#[test]
fn test_mixed() {
	setup_test!(
		"CARGO_CFG_TARGET_OS": "windows",
		"CARGO_CFG_TARGET_POINTER_WIDTH": "64",
		"CARGO_CFG_TARGET_FEATURE": "mmx,sse"
	);
	__test_basic();
	__test_set();
}

#[test]
fn test_target_family_shortcut() {
	setup_test!("CARGO_CFG_TARGET_FAMILY": "unix");
	assert!(build_cfg!(target_family = "unix"));
	assert!(!build_cfg!(target_family = "windows"));
	assert!(!build_cfg!(target_family = "wasm"));
	assert!(build_cfg!(unix));
	assert!(!build_cfg!(windows));
	assert!(!build_cfg!(wasm));

	setup_test!("CARGO_CFG_TARGET_FAMILY": "windows");
	assert!(!build_cfg!(target_family = "unix"));
	assert!(build_cfg!(target_family = "windows"));
	assert!(!build_cfg!(target_family = "wasm"));
	assert!(!build_cfg!(unix));
	assert!(build_cfg!(windows));
	assert!(!build_cfg!(wasm));

	setup_test!("CARGO_CFG_TARGET_FAMILY": "wasm");
	assert!(!build_cfg!(target_family = "unix"));
	assert!(!build_cfg!(target_family = "windows"));
	assert!(build_cfg!(target_family = "wasm"));
	assert!(!build_cfg!(unix));
	assert!(!build_cfg!(windows));
	assert!(build_cfg!(wasm));
}