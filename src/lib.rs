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
use std::io::Result;

/// Instruct cargo to set the VERSION environment variable to the version as
/// indicated by `git describe --tags --always --dirty=-modified`.
///
/// Also instructs cargo to *always* re-run the build script and recompile the
/// code, to make sure the version number is always correct.
pub fn set_env() {
	GitVersion::new()
		.set_env()
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
	GitVersion::new()
		.env_var_name(name.to_string())
		.set_env()
}

/// Same as `set_env_with_name`, but with explicit feedback about success and
/// failure.
///
/// If Err is returned, no environment variable is created.
/// If Ok is returned, cargo is instructed to set the environment variable
/// named `name` to is set to the version as
/// indicated by `git describe --tags --always --dirty=-modified`.
///
pub fn try_set_env_with_name(name: &str) -> Result<()> {
	GitVersion::new()
		.env_var_name(name.to_string())
		.try_set_env()
}


/// Notice: the combination of hash_length == 0 and long_format == true is illegal
pub struct GitVersion {
	hash_length: Option<usize>,
	dirty_suffix: Option<String>,
	broken_suffix: Option<String>,
	env_var_name: String,
	long_format: bool,
	unannotated_tags: bool,
	contains_tags: bool,
}

impl GitVersion {
	pub fn new() -> Self {
		GitVersion {
			hash_length: None,
			dirty_suffix: Some("-modified".to_string()),
			broken_suffix: None,
			env_var_name: "VERSION".to_string(),
			long_format: false,
			unannotated_tags: true,
			contains_tags: false,
		}
	}
	
	pub fn hash_length(&mut self, len: Option<usize>) -> &mut Self {
		self.hash_length = len;
		
		self
	}
	
	pub fn env_var_name(&mut self, name: String) -> &mut Self {
		self.env_var_name = name;
		
		self
	}
	
	// TODO add further confs
	
	pub fn set_env(&self) {
		self.set_env_with_default("undetermined");
	}
	
	pub fn set_env_with_default(&self, default_tag: &str) {
		if let Err(e) = self.try_set_env() {
			// Catch general error
			eprintln!("[git-version] Error: {}", e);
			
			println!("cargo:rustc-env={}={}", self.env_var_name, default_tag);
			println!("cargo:rerun-if-changed=(nonexistentfile)");
		}
	}
	
	///
	/// Try to set the environment variable.
	///
	/// # Panic
	///
	/// This function panics if `hash_length(Some(0))` and `long_format(true)` have
	/// been both specified.
	/// This is because `git` rejects this specific combination:
	/// ```
	/// fatal: --long is incompatible with --abbrev=0
	/// ```
	/// Consequently, this combination of option values must never be configured
	/// in the first place and is considered a programming error.
	///
	/// Notice, that this panic would render your project unable to been build.
	///
	///
	pub fn try_set_env(&self) -> Result<()> {
		// Notice a violation of this assertion is an programmatic error,
		// which shall be indicated by a panic, to be fixed by the programmer.
		// Otherwise, `git` would fail and potentially it would be handled
		// silently by the 'undetermined' tag.
		assert!(!(self.hash_length == Some(0) && self.long_format));
		
		// TODO implement using the config of self
		let cmd = Command::new("git").args(
		&["describe", "--always", "--tags", "--dirty=-modified"]).output()?;
		
		if !cmd.status.success() {
			return Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Git failed to describe HEAD, return code: {:?}\n{}",
					cmd.status.code(),
					String::from_utf8_lossy(&cmd.stderr)
				)
			));
		}
		
		let ver = String::from_utf8_lossy(&cmd.stdout);
		
		println!("cargo:rustc-env={}={}", self.env_var_name, ver);
		println!("cargo:rerun-if-changed=(nonexistentfile)");
		
		Ok(())
	}
}




