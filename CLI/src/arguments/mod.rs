use clap::{Args, Parser, Subcommand, ValueEnum};

use self::optimizations::{COMPILE_OUTPUT_NOTICE, OPTIMIZATION_GUIDE, OptimizationArguments};

mod optimizations;

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

#[derive(Clone, Copy, ValueEnum)]
pub enum RuntimeTarget {
	Luau,
	LuaJIT,
}

#[derive(Args)]
#[command(after_help = format!("{OPTIMIZATION_GUIDE}\n\n{COMPILE_OUTPUT_NOTICE}"))]
pub struct CompileArguments {
	/// Input source file
	pub file: String,

	/// Input source format
	#[arg(long, short, default_value = "web-assembly")]
	pub source: Source,

	/// Output target format
	#[arg(long, short, default_value = "luau")]
	pub target: Target,

	#[command(flatten)]
	pub optimizations: OptimizationArguments,
}

#[derive(Args)]
#[command(after_help = "The runtime script is written to stdout.")]
pub struct RuntimeArguments {
	/// Runtime target format
	#[arg(long, short, default_value = "luau")]
	pub target: RuntimeTarget,
}

#[derive(Subcommand)]
#[expect(
	clippy::large_enum_variant,
	reason = "the resolved compile policy is short-lived and does not justify an allocation"
)]
pub enum Command {
	/// Compile a source file
	#[command(long_about = format!("Compile a source file.\n\n{OPTIMIZATION_GUIDE}"))]
	Compile(CompileArguments),
	/// Print a complete runtime script
	Runtime(RuntimeArguments),
}

#[derive(Parser)]
#[command(
	version,
	about = "Compile source programs or print target runtime scripts."
)]
pub struct Arguments {
	#[command(subcommand)]
	pub command: Command,
}
