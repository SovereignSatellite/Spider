//! Build script for generating ISLE visitor code.

use std::path::Path;
use std::{env, fs};

use cranelift_isle::{codegen::CodegenOptions, compile::from_files};

static CODE_OPTIONS: CodegenOptions = CodegenOptions {
	exclude_global_allow_pragmas: true,
	prefixes: Vec::new(),
};

fn read_from_files(path: &Path) -> String {
	let files = fs::read_dir(path).unwrap().map(|file| file.unwrap().path());

	from_files(files, &CODE_OPTIONS).unwrap()
}

fn write_into_file(path: &Path, code: String) {
	let directory = env::var("OUT_DIR").unwrap();
	let path = Path::new(&directory).join(path);

	fs::write(path, code).unwrap();
}

fn fixup_generated_code(code: &str) -> String {
	code // Use `no_std` compatible paths.
		.replace("std::vec::Vec", "alloc::vec::Vec")
		.replace("std::marker", "core::marker")
		.replace("std::ops", "core::ops")
		// Remove imports superseded by the above or by the module prelude.
		.replace("use core::marker::PhantomData;\n", "")
		.replace("use super::*;  // Pulls in all external types.\n", "")
		// Prefer `Self` over the concrete type name.
		.replace("alloc::vec::Vec::len(self)", "Self::len(self)")
		.replace(
			"fn default() -> Self {\n        ContextIterWrapper {",
			"fn default() -> Self {\n        Self {",
		)
}

fn main() {
	println!("cargo:rerun-if-changed=isle");

	let code = read_from_files("isle/".as_ref());
	let code = fixup_generated_code(&code);

	write_into_file("isle.rs".as_ref(), code);
}
