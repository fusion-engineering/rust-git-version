use std::ffi::OsStr;
use std::path::{PathBuf, Path};
use std::process::Command;

/// Run `git describe` for the current working directory with custom flags to get version information from git.
pub fn describe<I, S>(dir: impl AsRef<Path>, args: I) -> Result<String, String>
where
	I: IntoIterator<Item = S>,
	S: AsRef<OsStr>,
{
	let dir = dir.as_ref();
	run_git("git describe", Command::new("git")
		.arg("-C")
		.arg(dir)
		.arg("describe").args(args))
}

/// Get the git directory for the given directory.
pub fn git_dir(dir: impl AsRef<Path>) -> Result<PathBuf, String> {
	let dir = dir.as_ref();
	let path = run_git("git rev-parse", Command::new("git")
		.arg("-C")
		.arg(dir)
		.args(["rev-parse", "--git-dir"]))?;
	Ok(dir.join(path))
}

/// Run `git submodule foreach` command to discover submodules in the project.
pub fn get_submodules(dir: impl AsRef<Path>) -> Result<Vec<String>, String> {
	let dir = dir.as_ref();
	let result = run_git("git submodule",
		Command::new("git")
			.arg("-C")
			.arg(dir)
			.arg("submodule")
			.arg("foreach")
			.arg("--quiet")
			.arg("--recursive")
			.arg("echo $displaypath"),
	)?;

	Ok(result.lines()
		.filter(|x| !x.is_empty())
		.map(|x| x.to_owned())
		.collect()
	)
}

fn run_git(program: &str, command: &mut std::process::Command) -> Result<String, String> {
	let output = command
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn()
		.map_err(|e| {
			if e.kind() == std::io::ErrorKind::NotFound {
				format!("Command `{}` not found: is git installed?", command.get_program().to_string_lossy())
			} else {
				format!("Failed to run `{}`: {}", command.get_program().to_string_lossy(), e)
			}
		})?
		.wait_with_output()
		.map_err(|e| format!("Failed to wait for `{}`: {}", program, e))?;

	let output = collect_output(program, output)?;
	let output = strip_trailing_newline(output);
	let output =
		String::from_utf8(output).map_err(|_| format!("Failed to parse output of `{}`: output contains invalid UTF-8", program))?;
	Ok(output)
}

/// Check if a command ran successfully, and if not, return a verbose error.
fn collect_output(program: &str, output: std::process::Output) -> Result<Vec<u8>, String> {
	// If the command succeeded, just return the output as is.
	if output.status.success() {
		return Ok(output.stdout);

	// If the command terminated with non-zero exit code, return an error.
	} else if let Some(status) = output.status.code() {
		// Include the first line of stderr in the error message, if it's valid UTF-8 and not empty.
		let message = output
			.stderr
			.split(|c| *c == b'\n')
			.next()
			.and_then(|x| std::str::from_utf8(x).ok())
			.filter(|x| !x.is_empty());
		if let Some(message) = message {
			return Err(format!("{} exited with status {}: {}", program, status, message));
		} else {
			return Err(format!("{} exited with status {}", program, status));
		}
	}

	// The command was killed by a signal.
	#[cfg(unix)]
	{
		use std::os::unix::process::ExitStatusExt;
		if let Some(signal) = output.status.signal() {
			// Include the signal number on Unix.
			return Err(format!("{} killed by signal {}", program, signal));
		}
	}

	Err(format!("{} exitted with error", program))
}

/// Remove a trailing newline from a byte string.
fn strip_trailing_newline(mut input: Vec<u8>) -> Vec<u8> {
	if input.last().copied() == Some(b'\n') {
		input.pop();
	}
	input
}

#[test]
fn test_git_dir() {
	use assert2::{assert, let_assert};
	use std::path::Path;

	let_assert!(Ok(git_dir) = git_dir("."));
	let_assert!(Ok(git_dir) = git_dir.canonicalize());
	let_assert!(Ok(expected) = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.git").canonicalize());
	assert!(git_dir == expected);
}
