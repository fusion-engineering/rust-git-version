//! Use this library in your `build.rs` script:
//!
//! ```
//! extern crate git_version;
//! fn main() { git_version::set_env(); }
//! ```
//!
//! Then you can use `env!("VERSION")` to get the version number in your code.
//! The version number will be based on the relevant git tag (if any), and git
//! commit hash if there is no exactly matching tag. See `git help describe`.
//!
//! The version number will have a `-modified` suffix if your git worktree had
//! untracked or changed files.

use std::process::Command;

/// Instruct cargo to set the VERSION environment variable to the version as
/// indicated by `git describe --always --dirty=-modified`.
///
/// Also instructs cargo to *always* re-run the build script and recompile the
/// code, to make sure the version number is always correct.
pub fn set_env() {
	let cmd = Command::new("git").args(&["describe", "--always", "--dirty=-modified"]).output().unwrap();
	assert!(cmd.status.success());
	let ver = std::str::from_utf8(&cmd.stdout[..]).unwrap().trim();
	println!("cargo:rustc-env=VERSION={}", ver);
	println!("cargo:rerun-if-changed=(nonexistentfile)");
}
