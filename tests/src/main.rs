fn main() {
	panic!("Use `cargo test`");
}

#[cfg_attr(test, macro_use)]
extern crate build_cfg;

#[cfg(test)]
mod tests;