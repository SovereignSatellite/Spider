use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, ValueEnum)]
pub enum Source {
	TuringMachine,
	WebAssembly,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Target {
	Json,
	Luau,
	LuaJIT,
}

#[derive(Parser)]
#[command(version)]
pub struct Arguments {
	/// The source file for processing
	pub file: String,

	/// The format of the source file
	#[arg(long, short, default_value = "web-assembly")]
	pub source: Source,

	/// The format of the output file
	#[arg(long, short, default_value = "luau")]
	pub target: Target,

	/// Run all optimization passes
	#[arg(long, short)]
	pub should_optimize: bool,
}
