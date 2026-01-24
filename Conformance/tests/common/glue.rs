use std::{
	ffi::OsStr,
	io::{Read, Result},
	path::{Path, PathBuf},
	process::{Child, Command, ExitStatus, Stdio},
	time::{Duration, Instant},
};

pub fn enable_back_trace() {
	// SAFETY: I'm not sure, but it's not a problem in practice.
	unsafe {
		std::env::set_var("RUST_BACKTRACE", "1");
	}
}

fn poll_until_timeout(child: &mut Child, duration: Duration) -> Result<ExitStatus> {
	let now = Instant::now();

	while now.elapsed() < duration {
		std::thread::yield_now();

		if let Some(status) = child.try_wait()? {
			return Ok(status);
		}
	}

	child.kill()?;

	Err(std::io::Error::new(
		std::io::ErrorKind::TimedOut,
		"the sub-process has timed out",
	))
}

fn fmt_process_output(child: Child, out: &mut String) -> Result<()> {
	let Child { stdout, stderr, .. } = child;

	out.push_str("\nLUAU STANDARD ERROR\n");
	stderr.unwrap().read_to_string(out)?;

	out.push_str("\nLUAU STANDARD OUTPUT\n");
	stdout.unwrap().read_to_string(out)?;

	Ok(())
}

pub fn get_path_target(extension: &OsStr, name: &OsStr) -> PathBuf {
	const TEMP_DIRECTORY: &str = env!("CARGO_TARGET_TMPDIR");

	Path::new(TEMP_DIRECTORY)
		.join(name)
		.with_extension(extension)
}

pub fn run(path: &OsStr, arguments: &[&OsStr]) -> Result<()> {
	const TEST_TIMEOUT: Duration = Duration::from_secs(1);

	let mut child = Command::new(path)
		.args(arguments)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	let mut output = match poll_until_timeout(&mut child, TEST_TIMEOUT) {
		Ok(status) if status.success() => return Ok(()),
		Ok(_) => String::new(),
		Err(error) => error.to_string(),
	};

	fmt_process_output(child, &mut output)?;

	if output.is_empty() {
		Ok(())
	} else {
		panic!("{output}");
	}
}
