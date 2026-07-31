use clap::{
	Arg, ArgMatches, Args, Command as ClapCommand, Error, FromArgMatches, ValueEnum,
	builder::BoolishValueParser,
};

use ir_passes::catalog::{
	Optimization, OptimizationDomain, OptimizationIntent, OptimizationLevel, Optimizations,
};
use ir_pipeline::OptimizationConfiguration;

/// Provide the optimization-selection guide shared by compile help.
pub const OPTIMIZATION_GUIDE: &str = "Optimization selection:
  More specific selectors override broader selectors: individual optimizations override intents,
  intents override domains, and domains override the preset, regardless of argument order.
  Use --flag or --flag=true to enable a domain, intent, or individual optimization; use
  --flag=false to disable it. The default is -O0.
  When supported by the selected target, optional target-lowering optimizations replace
  intermediate operations with target-specific equivalents.";

/// Provide the compile-output notice shared by short and long help.
pub const COMPILE_OUTPUT_NOTICE: &str = "The compiled output is written to stdout.";

const OPTIMIZATION_HEADING: &str = "Optimization levels and limits";
const DOMAIN_HEADING: &str = "Optimization domains";
const INTENT_HEADING: &str = "Optimization intents";
const OPTIMIZATION_LEVEL_ARGUMENT: &str = "optimization-level";
const OPTIMIZATION_ROUND_LIMIT_ARGUMENT: &str = "optimization-round-limit";

fn selector_argument(flag: &'static str, help: &'static str) -> Arg {
	Arg::new(flag)
		.long(flag)
		.num_args(0..=1)
		.require_equals(true)
		.default_missing_value("true")
		.value_name("BOOL")
		.value_parser(BoolishValueParser::new())
		.hide_possible_values(true)
		.help(help)
}

fn detailed_selector_argument(
	flag: &'static str,
	help: &'static str,
	heading: &'static str,
) -> Arg {
	selector_argument(flag, help)
		.help_heading(heading)
		.hide_short_help(true)
}

fn add_detailed_optimization_arguments(mut command: ClapCommand) -> ClapCommand {
	for &domain in OptimizationDomain::ALL {
		for &intent in OptimizationIntent::ALL {
			if intent.domain() == domain {
				command = command.arg(detailed_selector_argument(
					intent.flag(),
					intent.help(),
					INTENT_HEADING,
				));
			}
		}
	}

	for &domain in OptimizationDomain::ALL {
		for &optimization in Optimization::ALL {
			let descriptor = optimization.descriptor();

			if descriptor.intent.domain() == domain {
				command = command.arg(detailed_selector_argument(
					descriptor.flag,
					descriptor.help,
					domain.exact_help_heading(),
				));
			}
		}
	}

	command
}

fn add_optimization_arguments(mut command: ClapCommand) -> ClapCommand {
	command = command
		.after_long_help(COMPILE_OUTPUT_NOTICE)
		.arg(
			Arg::new(OPTIMIZATION_LEVEL_ARGUMENT)
				.short('O')
				.long(OPTIMIZATION_LEVEL_ARGUMENT)
				.value_name("LEVEL")
				.value_parser(clap::value_parser!(OptimizationPreset))
				.default_value("0")
				.help_heading(OPTIMIZATION_HEADING)
				.help("Select the cumulative -O0, -O1, -O2, or -O3 optimization preset."),
		)
		.arg(
			Arg::new(OPTIMIZATION_ROUND_LIMIT_ARGUMENT)
				.long(OPTIMIZATION_ROUND_LIMIT_ARGUMENT)
				.value_name("ROUNDS")
				.value_parser(clap::value_parser!(u32))
				.help_heading(OPTIMIZATION_HEADING)
				.help(
					"Limit each graph-changing fixpoint phase to this many rounds; omitted means 4,294,967,295 rounds.",
				),
		);

	for &domain in OptimizationDomain::ALL {
		command = command
			.arg(selector_argument(domain.flag(), domain.help()).help_heading(DOMAIN_HEADING));
	}

	add_detailed_optimization_arguments(command)
}

#[derive(Clone, Copy, ValueEnum)]
enum OptimizationPreset {
	#[value(name = "0")]
	Zero,
	#[value(name = "1")]
	One,
	#[value(name = "2")]
	Two,
	#[value(name = "3")]
	Three,
}

impl OptimizationPreset {
	const fn contains(self, level: OptimizationLevel) -> bool {
		match self {
			Self::Zero => false,
			Self::One => matches!(level, OptimizationLevel::One),
			Self::Two => matches!(level, OptimizationLevel::One | OptimizationLevel::Two),
			Self::Three => true,
		}
	}
}

fn optimization_configuration(matches: &ArgMatches) -> OptimizationConfiguration {
	let preset = *matches
		.get_one::<OptimizationPreset>(OPTIMIZATION_LEVEL_ARGUMENT)
		.unwrap();
	let mut optimizations = Optimizations::none();

	for &optimization in Optimization::ALL {
		let descriptor = optimization.descriptor();
		let enabled = matches
			.get_one::<bool>(descriptor.flag)
			.copied()
			.or_else(|| matches.get_one::<bool>(descriptor.intent.flag()).copied())
			.or_else(|| {
				matches
					.get_one::<bool>(descriptor.intent.domain().flag())
					.copied()
			})
			.unwrap_or_else(|| preset.contains(descriptor.level));

		if enabled {
			optimizations.enable(optimization);
		}
	}

	let round_limit = matches
		.get_one::<u32>(OPTIMIZATION_ROUND_LIMIT_ARGUMENT)
		.copied()
		.unwrap_or(u32::MAX);

	OptimizationConfiguration {
		optimizations,
		round_limit,
	}
}

/// Parse optimization selectors into one resolved configuration.
pub struct OptimizationArguments {
	/// Store the resolved configuration.
	pub configuration: OptimizationConfiguration,
}

impl FromArgMatches for OptimizationArguments {
	fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Error> {
		Ok(Self {
			configuration: optimization_configuration(matches),
		})
	}

	fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), Error> {
		*self = Self::from_arg_matches(matches)?;

		Ok(())
	}
}

impl Args for OptimizationArguments {
	fn augment_args(command: ClapCommand) -> ClapCommand {
		add_optimization_arguments(command)
	}

	fn augment_args_for_update(command: ClapCommand) -> ClapCommand {
		Self::augment_args(command)
	}
}
