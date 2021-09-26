use build_cfg_shared::CfgKey;
use proc_macro::TokenStream;

use quote::quote;
use syn::{ItemFn, Lit, LitStr, Meta, MetaNameValue, NestedMeta, PathArguments, PathSegment, parse_macro_input, punctuated::Punctuated, spanned::Spanned};

#[proc_macro_attribute]
/// This needs to be added to your `fn main()` function in your build script.
pub fn build_cfg_main(_: TokenStream, main_fn: TokenStream) -> TokenStream {
	let mut main_fn = parse_macro_input!(main_fn as ItemFn);
	assert!(main_fn.sig.ident.to_string() == "main", "This is not the main function of the build script");
	main_fn.block.stmts.insert(0, syn::parse_str("::build_cfg::__private::populate_cfg(std::env::vars_os());").unwrap());
	quote!(#main_fn).into()
}

#[proc_macro]
/// Tests `cfg` directives at build time.
///
/// ## Example
///
/// ```rust
/// #[macro_use]
/// extern crate build_cfg;
///
/// #[build_cfg_main]
/// fn main() {
///     if build_cfg!(all(target_os = "windows", target_pointer_width = "64")) {
///	        println!("Building for Windows 64-bit");
///     } else {
///	        println!("Not building for Windows 64-bit");
///     }
/// }
/// ```
///
/// # Panics
///
/// This function will panic if `#[build_cfg_main]` has not been added to the `fn main()` function of your build script, or was otherwise unable to get the `cfg` directives from the environment variables set by Cargo.
pub fn build_cfg(cfg: TokenStream) -> TokenStream {
	let cfg = parse_macro_input!(cfg as Meta);

	fn parse_cfg(cfg: Meta, mut output: String) -> String {
		use std::fmt::Write;
		match cfg {
			Meta::Path(ref path) => {
				match path.get_ident() {
					Some(ident) => {
						let target_family = ident.to_string();
						if !matches!(target_family.as_str(), "windows" | "unix" | "wasm") {
							panic!("Unknown cfg directive: `{}`", target_family);
						};
						output = parse_cfg(Meta::NameValue(MetaNameValue {
							lit: Lit::Str(LitStr::new(target_family.as_str(), cfg.span())),
							path: syn::Path {
								leading_colon: None,
								segments: {
									let mut segments = Punctuated::new();
									segments.push_value(PathSegment {
										ident: proc_macro2::Ident::new("target_family", cfg.span()),
										arguments: PathArguments::None
									});
									segments
								}
							},
							eq_token: Default::default(),
						}), output);
					},
					None => panic!("`cfg` predicate key must be an identifier")
				}
			},

			Meta::List(list) => {
				assert!(list.path.segments.len() == 1, "Syntax error");
				let predicate = list.path.segments.first().expect("Missing predicate").ident.to_string();
				match predicate.as_str() {
					predicate @ ("any" | "all") => {
						match predicate {
							"any" => output.push_str("::build_cfg::__private::CfgPredicate::Any(vec!["),
							"all" => output.push_str("::build_cfg::__private::CfgPredicate::All(vec!["),
							_ => unreachable!()
						}
						for meta in list.nested {
							let meta = match meta {
								NestedMeta::Meta(meta) => meta,
								NestedMeta::Lit(_) => panic!("unsupported literal"),
							};
							output = parse_cfg(meta, output);
							output.push(',');
						}
						output.pop();
						output.push_str("])");
					},
					"not" => {
						output.push_str("::build_cfg::__private::CfgPredicate::Not(Box::new(");
						for meta in list.nested {
							match meta {
								NestedMeta::Meta(meta) => output = parse_cfg(meta, output),
								NestedMeta::Lit(_) => panic!("unsupported literal"),
							}
						}
						output.push_str("))");
					},
					_ => panic!("Unknown predicate (`{}`)", predicate)
				}
			},

			Meta::NameValue(nv) => {
				let key = nv.path.segments[0].ident.to_string();
				let key = match CfgKey::from_str(&key) {
					Some(key) => key,
					None => panic!("Unknown cfg directive: `{}`", key)
				};
				let val = match nv.lit {
					syn::Lit::Str(str) => str.value(),
					_ => panic!("literal in `cfg` predicate value must be a string")
				};
				write!(output, "::build_cfg::__private::CfgPredicate::Directive(::build_cfg::__private::CfgDirective(::build_cfg::__private::CfgKey::{:?},{:?}))", key, val).ok();
			},
		}
		output
	}

	let mut output = parse_cfg(cfg, String::new());
	output.push_str(".test()");
	output.parse().unwrap()
}