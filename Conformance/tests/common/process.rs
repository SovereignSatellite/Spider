use core::time::Duration;
use std::{
	ffi::OsStr,
	io::{self, Read as _, Result},
	process::{Child, Command, ExitStatus, Stdio},
	thread,
	time::Instant,
};

fn poll_until_timeout(child: &mut Child, duration: Duration) -> Result<ExitStatus> {
	let start = Instant::now();

	while start.elapsed() < duration {
		thread::sleep(Duration::from_millis(100));

		if let Some(status) = child.try_wait()? {
			return Ok(status);
		}
	}

	child.kill()?;

	Err(io::Error::new(
		io::ErrorKind::TimedOut,
		"the sub-process has timed out",
	))
}

fn push_all_output(child: Child, out: &mut String) -> Result<()> {
	let Child { stdout, stderr, .. } = child;

	out.push_str("\nTEST STANDARD ERROR\n");
	stderr.unwrap().read_to_string(out)?;

	out.push_str("\nTEST STANDARD OUTPUT\n");
	stdout.unwrap().read_to_string(out)?;

	Ok(())
}

pub fn run(path: &OsStr, arguments: &[&OsStr]) -> Result<Box<str>> {
	const TEST_TIMEOUT: Duration = Duration::from_secs(4);

	let mut child = Command::new(path)
		.args(arguments)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	let mut output = match poll_until_timeout(&mut child, TEST_TIMEOUT) {
		Ok(status) if status.success() => return Ok(Box::default()),

		Ok(status) => status.to_string(),
		Err(error) => error.to_string(),
	};

	push_all_output(child, &mut output)?;

	Ok(output.into_boxed_str())
}
