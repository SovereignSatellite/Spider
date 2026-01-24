use std::{
	ffi::OsStr,
	io::{Read, Result},
	path::{Path, PathBuf},
	process::{Child, Command, ExitStatus, Stdio},
	time::{Duration, Instant},
};

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

fn fail_with_output(child: Child) -> Result<()> {
	let Child { stdout, stderr, .. } = child;

	let mut result = String::new();

	stdout.unwrap().read_to_string(&mut result)?;

	if !result.is_empty() {
		result.push('\n');
	}

	stderr.unwrap().read_to_string(&mut result)?;

	panic!("{result}");
}

pub fn get_path_target(extension: &OsStr, name: &OsStr) -> PathBuf {
	const TEMP_DIRECTORY: &str = env!("CARGO_TARGET_TMPDIR");

	Path::new(TEMP_DIRECTORY)
		.join(name)
		.with_extension(extension)
}

pub fn run(path: &OsStr, source: &OsStr) -> Result<()> {
	const TEST_TIMEOUT: Duration = Duration::from_secs(1);

	let mut child = Command::new(path)
		.arg(source)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	if poll_until_timeout(&mut child, TEST_TIMEOUT)?.success() {
		Ok(())
	} else {
		fail_with_output(child)
	}
}
