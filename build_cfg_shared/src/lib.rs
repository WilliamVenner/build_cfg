#[macro_use]
extern crate derive_more;

pub trait CfgTest {
	fn test(&self, val: &'static str) -> bool;
	fn put(&mut self, val: String);
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct CfgSet(std::collections::BTreeSet<String>);
impl CfgTest for CfgSet {
	#[inline]
	fn test(&self, val: &'static str) -> bool {
		self.contains(val)
	}

	#[inline]
	fn put(&mut self, val: String) {
		for val in val.split(",") {
			self.insert(val.to_string());
		}
	}
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct CfgScalar(String);
impl CfgTest for CfgScalar {
	#[inline]
	fn test(&self, val: &'static str) -> bool {
		self.as_str() == val
	}

	#[inline]
	fn put(&mut self, val: String) {
		self.0 = val;
	}
}
impl CfgTest for Option<CfgScalar> {
	#[inline]
	fn test(&self, val: &'static str) -> bool {
		if let Some(ref _self) = self {
			_self.test(val)
		} else {
			false
		}
	}

	#[inline]
	fn put(&mut self, val: String) {
		self.replace(CfgScalar(val));
	}
}

#[derive(Debug, Default)]
pub struct EnvCfg {
	pub target_os: CfgScalar,
	pub target_arch: CfgScalar,
	pub target_endian: CfgScalar,
	pub target_pointer_width: CfgScalar,
	pub target_family: Option<CfgScalar>,
	pub target_env: CfgScalar,
	pub target_vendor: Option<CfgScalar>,
	pub target_has_atomic: CfgSet,
	pub target_feature: CfgSet,
}

macro_rules! cfg_key {
	{ $($variant:ident = $val:ident),* } => {
		#[derive(Clone, Copy, Debug)]
		pub enum CfgKey {
			$($variant),*
		}
		impl CfgKey {
			pub fn lookup(self, cfg: &EnvCfg) -> &dyn CfgTest {
				match self {
					$(CfgKey::$variant => &cfg.$val),*
				}
			}

			pub fn lookup_mut(self, cfg: &mut EnvCfg) -> &mut dyn CfgTest {
				match self {
					$(CfgKey::$variant => &mut cfg.$val),*
				}
			}

			pub fn from_str<S: AsRef<str>>(str: S) -> Option<CfgKey> {
				match str.as_ref() {
					$(stringify!($val) => Some(CfgKey::$variant)),*,
					_ => None
				}
			}
		}
	};
}
cfg_key! {
	TargetOs = target_os,
	TargetArch = target_arch,
	TargetEndian = target_endian,
	TargetPointerWidth = target_pointer_width,
	TargetFamily = target_family,
	TargetEnv = target_env,
	TargetVendor = target_vendor,
	TargetHasAtomic = target_has_atomic,
	TargetFeature = target_feature
}

#[derive(Clone, Copy, Debug)]
pub struct CfgDirective(pub CfgKey, pub &'static str);

#[derive(Debug)]
pub enum CfgPredicate {
	Any(Vec<CfgPredicate>),
	All(Vec<CfgPredicate>),
	Not(Box<CfgPredicate>),
	Directive(CfgDirective),
}
impl CfgPredicate {
	pub fn test(self) -> bool {
		match self {
			CfgPredicate::Any(directives) => {
				for predicate in directives {
					if predicate.test() {
						return true;
					}
				}
				false
			}
			CfgPredicate::All(directives) => {
				for predicate in directives {
					if !predicate.test() {
						return false;
					}
				}
				true
			}
			CfgPredicate::Not(predicate) => !predicate.test(),
			CfgPredicate::Directive(directive) => env_cfg(move |env_cfg| directive.0.lookup(env_cfg).test(directive.1)),
		}
	}
}

thread_local! {
	pub static __TARGET_CFG: std::cell::RefCell<Option<EnvCfg>> = std::cell::RefCell::new(None);
}
#[inline]
pub fn env_cfg<T, F>(f: F) -> T
where
	F: FnOnce(&EnvCfg) -> T + 'static
{
	__TARGET_CFG.with(|target_cfg| {
		f(target_cfg.borrow().as_ref().expect("build_cfg has not been initialized with the build script's environment variables. You probably forgot to add `#[build_cfg_main]` to your `fn main()` function."))
	})
}
