//! Use this library in your `build.rs` script:
//!
//! ```
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

use std::process::exit;
use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use git_version_macro::git_describe;

#[proc_macro_hack]
pub use git_version_macro::git_version;

pub use git_version_impl::{
	describe,
	describe_cwd,
	version,
	version_cwd,
};

/// Instruct cargo to set the VERSION environment variable to the version as
/// indicated by `git describe --always --dirty=-modified`.
///
/// Also instructs cargo to *always* re-run the build script and recompile the
/// code, to make sure the version number is always correct.
pub fn set_env() {
	set_env_with_name("VERSION");
}

/// Same as `set_env`, but using `name` as the name for the environment
/// variable.
///
/// You can, for example, override the `CARGO_PKG_VERSION` using in
/// your `build.rs` script:
///
/// ```
/// fn main() { git_version::set_env_with_name("CARGO_PKG_VERSION"); }
/// ```
pub fn set_env_with_name(name: &str) {
	if let Err(e) = try_set_env_with_name(name) {
		eprintln!("[git-version] Error: {}", e);
		exit(1);
	}
}

/// Same as `set_env_with_name`, but with explicit feedback about success and
/// failure.
///
/// If `Err` is returned, no environment variable is created.
/// If `Ok` is returned, cargo is instructed to set the environment variable
/// named `name` to is set to the version as indicated by
/// `git describe --always --dirty=-modified`.
pub fn try_set_env_with_name(name: &str) -> std::io::Result<()> {
	let version = version_cwd()?;

	println!("cargo:rustc-env={}={}", name, version);
	println!("cargo:rerun-if-changed=(nonexistentfile)");

	Ok(())
}