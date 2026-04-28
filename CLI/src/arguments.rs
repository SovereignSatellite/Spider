use clap::{Args, Parser, Subcommand, ValueEnum};

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
#[command(after_help = "The compiled output is written to stdout.")]
pub struct CompileArguments {
	/// Input source file
	pub file: String,

	/// Input source format
	#[arg(long, short, default_value = "web-assembly")]
	pub source: Source,

	/// Output target format
	#[arg(long, short, default_value = "luau")]
	pub target: Target,

	/// Run optimization passes before printing
	#[arg(long, short = 'O')]
	pub optimize: bool,
}

#[derive(Args)]
#[command(after_help = "The runtime script is written to stdout.")]
pub struct RuntimeArguments {
	/// Runtime target format
	#[arg(long, short, default_value = "luau")]
	pub target: RuntimeTarget,
}

#[derive(Subcommand)]
pub enum Command {
	/// Compile a source file
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
