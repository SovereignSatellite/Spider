use std::path::Path;

use cranelift_isle::{codegen::CodegenOptions, compile::from_files};

static CODE_OPTIONS: CodegenOptions = CodegenOptions {
	exclude_global_allow_pragmas: true,
	prefixes: Vec::new(),
};

fn read_from_files(path: &Path) -> String {
	let files = std::fs::read_dir(path)
		.unwrap()
		.map(|file| file.unwrap().path());

	from_files(files, &CODE_OPTIONS).unwrap()
}

fn write_into_file(path: &Path, code: String) {
	let directory = std::env::var("OUT_DIR").unwrap();
	let path = Path::new(&directory).join(path);

	std::fs::write(path, code).unwrap();
}

fn main() {
	println!("cargo:rerun-if-changed=isle");

	let code = read_from_files("isle/".as_ref());

	write_into_file("isle.rs".as_ref(), code);
}
