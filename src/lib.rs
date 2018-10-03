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

/// Represents the three different formats of a git version.
///
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum GitVersionFormat {
	/// Take only the tag string of the closest git tag omitting the
	/// commit hash.
	TagOnly,
	
	/// The default output format of `git describe`,
	/// which takes only the tag if the commit of `HEAD` is directly tagged
	/// or the `Long` format otherwise.
	Fancy,
	
	/// Uses always the long format:
	/// ```
	/// format!("{}-{}-g{}", tag_name, commits_ahead, commit_hash)
	/// ```
	Long,
}


/// A git version builder, providing fine-grain control over the resulting
/// version name.
///
/// A minimal usage version is `GitVersion::new().set_env()`
///
#[derive(Clone,Debug)]
pub struct GitVersion {
	/// The name of the environment variable to be set.
	env_var_name: String,
	
	/// If Some, add the given suffix when the working tree differs from HEAD.
	dirty_suffix: Option<String>,
	
	/// If Some, add the given suffix when the state of the working tree is
	/// indeterminable.
	broken_suffix: Option<String>,
	
	/// The number of characters in the hash.
	hash_length: usize,
	
	/// The number of tags to consider when searching for a tag
	candidates: usize,
	
	/// The output format of the description.
	format: GitVersionFormat,
	
	/// Include non-annotated tags
	unannotated_tags: bool,
	
	/// Reverse the order of searching tags.
	contains_tags: bool,
	
	/// Travers only the first parent at merge commits.
	first_parent: bool,
	
	/// Allow all kinds of reference as tag
	all_refs: bool,
}

impl GitVersion {
	pub fn new() -> Self {
		GitVersion {
			env_var_name: "VERSION".to_string(),
			dirty_suffix: Some("-modified".to_string()),
			broken_suffix: Some("-broken".to_string()),
			hash_length: 0,
			candidates: 10,
			format: GitVersionFormat::Fancy,
			unannotated_tags: true,
			contains_tags: false,
			first_parent: false,
			all_refs: false,
		}
	}
	
	pub fn env_var_name(&mut self, name: String) -> &mut Self {
		self.env_var_name = name;
		
		self
	}
	
	pub fn dirty_suffix(&mut self, suffix: Option<String>) -> &mut Self {
		self.dirty_suffix = suffix;
		
		self
	}
	
	pub fn broken_suffix(&mut self, suffix: Option<String>) -> &mut Self {
		self.broken_suffix = suffix;
		
		self
	}
	
	pub fn hash_length(&mut self, len: usize) -> &mut Self {
		self.hash_length = len;
		
		self
	}
	
	pub fn candidates(&mut self, number: usize) -> &mut Self {
		self.candidates = number;
		
		self
	}
	
	pub fn format(&mut self, fmt: GitVersionFormat) -> &mut Self {
		self.format = fmt;
		
		self
	}
	
	pub fn unannotated_tags(&mut self, val: bool) -> &mut Self {
		self.unannotated_tags = val;
		
		self
	}
	
	pub fn contains_tags(&mut self, val: bool) -> &mut Self {
		self.contains_tags = val;
		
		self
	}
	
	pub fn first_parent(&mut self, val: bool) -> &mut Self {
		self.first_parent = val;
		
		self
	}
	
	pub fn all_refs(&mut self, val: bool) -> &mut Self {
		self.all_refs = val;
		
		self
	}
	
	pub fn set_env(&self) {
		self.set_env_with_default("undetermined");
	}
	
	pub fn set_env_with_default(&self, default_tag: &str) {
		if let Err(e) = self.try_set_env() {
			// Catch general error
			// This error message can be displayed e.g. with:
			// cargo build -vv 2>&1 | grep -A4 "\\[git-version\\]"
			eprintln!("[git-version] Error: {}", e);
			
			println!("cargo:rustc-env={}={}", self.env_var_name, default_tag);
			println!("cargo:rerun-if-changed=(nonexistentfile)");
		}
	}
	
	
	/// Try to set the environment variable.
	///
	///
	pub fn try_set_env(&self) -> Result<()> {
		// Notice: the combination of --abbrev=0 and --long is illegal
		
		// Construct base command
		let mut cmd = Command::new("git");
		cmd.args(&[
			"describe",
			"--always",
			&format!("--candidates={}", self.candidates)
		]);
		
		// Add suffixes
		if let Some(ref suffix) = self.dirty_suffix {
			cmd.arg(&format!("--dirty={}", suffix));
		}
		if let Some(ref suffix) = self.broken_suffix {
			cmd.arg(&format!("--broken={}", suffix));
		}
		
		// Set format
		if self.format == GitVersionFormat::TagOnly {
			cmd.arg("--abbrev=0");
		} else if self.hash_length > 0 {
			cmd.arg(&format!("--abbrev={}", self.hash_length));
		}
		if self.format == GitVersionFormat::Long {
			cmd.arg("--long");
		}
		// Notice: self.format == GitVersionFormat::Fancy
		// is default behaviour and doesn't need handling
		
		// Set flags
		if self.unannotated_tags {
			cmd.arg("--tags");
		}
		if self.contains_tags {
			cmd.arg("--contains");
		}
		if self.first_parent {
			cmd.arg("--first-parent");
		}
		if self.all_refs {
			cmd.arg("--all");
		}
		
		// Start process and gather the stdout/stderr
		let cmd_res = cmd.output()?;
		
		// Check status code
		if !cmd_res.status.success() {
			return Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Git failed to describe the working tree, return code: {:?}\n{}",
					cmd_res.status.code(),
					String::from_utf8_lossy(&cmd_res.stderr)
				)
			));
		}
		
		// Convert the stdout of the process into UTF-8, non-UTF-8 bytes become
		// the `�` symbole
		let ver = String::from_utf8_lossy(&cmd_res.stdout);
		
		// Output the instructions for cargo to set the environment variable
		println!("cargo:rustc-env={}={}", self.env_var_name, ver);
		println!("cargo:rerun-if-changed=(nonexistentfile)");
		
		Ok(())
	}
}




