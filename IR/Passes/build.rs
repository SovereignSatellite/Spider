//! Build script for generating ISLE visitor code.

use std::{
	env, fs,
	path::{Path, PathBuf},
};

use cranelift_isle::{codegen::CodegenOptions, compile::from_files};

static CODE_OPTIONS: CodegenOptions = CodegenOptions {
	exclude_global_allow_pragmas: true,
	prefixes: Vec::new(),
	emit_logging: false,
	split_match_arms: false,
	match_arm_split_threshold: None,
};

fn collect_isle_files(directory: &Path, files: &mut Vec<PathBuf>) {
	for entry in fs::read_dir(directory).unwrap() {
		let path = entry.unwrap().path();

		if path.is_dir() {
			collect_isle_files(&path, files);
		} else {
			files.push(path);
		}
	}
}

fn read_from_files(path: &Path) -> String {
	let mut files = Vec::new();

	collect_isle_files(path, &mut files);

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
