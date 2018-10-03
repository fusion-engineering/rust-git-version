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
//!
//! Does not depend on libgit, but simply uses the `git` binary directly.
//! So you must have `git` installed somewhere in your `PATH`.

use std::process::Command;

/// Instruct cargo to set the VERSION environment variable to the version as
/// indicated by `git describe --always --dirty=-modified`.
///
/// Also instructs cargo to *always* re-run the build script and recompile the
/// code, to make sure the version number is always correct.
pub fn set_env() {
	set_env_with_name("VERSION");
}

/// Same as `set_env`, but using `name` as environment variable.
///
/// You can, for example, override the `CARGO_PKG_VERSION` using in
/// your `build.rs` script:
///
/// ```
/// extern crate git_version;
/// fn main() { git_version::set_env_with_name("CARGO_PKG_VERSION"); }
/// ```
pub fn set_env_with_name(name: &str) {
	let cmd = Command::new("git").args(&["describe", "--always", "--tags", "--dirty=-modified"]).output().unwrap();
	assert!(cmd.status.success());
	let ver = std::str::from_utf8(&cmd.stdout[..]).unwrap().trim();
	println!("cargo:rustc-env={}={}", name, ver);
	println!("cargo:rerun-if-changed=(nonexistentfile)");
}
