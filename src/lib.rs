//! Use this library in your `build.rs` script:
//!
//! ```
//! extern crate git_version;
//! fn main() { git_version::set_env(); }
//! ```
//!
//! Which requires in the `Cargo.toml`:
//! ```toml
//! [build-dependencies]
//! git-version = "*"
//! ```
//!
//! Then you can use `env!("VERSION")` to get the version number in your code.
//! The version number will be based on the relevant git tag (if any), and git
//! commit hash if there is no exactly matching tag. See `git help describe`.
//!
//! The version number will have a `-modified` suffix if your git working tree
//! had untracked or changed files.
//!
//! Does not depend on libgit, but simply uses the `git` binary directly.
//! So you must have `git` installed somewhere in your `PATH`.
//!
//! # Builder
//!
//! This library also has a builder-pattern like constructor [GitVersion],
//! which allows for fine-grain configuration of getting the version tag.
//! [GitVersion] remaps most of the capability of `git describe`.

use std::process::Command;
use std::io::Result;

/// Instruct cargo to set the VERSION environment variable to the version as
/// indicated by `git describe --always --dirty=-modified`.
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
/// If `Err` is returned, no environment variable is created.
/// If `Ok` is returned, cargo is instructed to set the environment variable
/// named `name` to is set to the version as
/// indicated by `git describe --always --dirty=-modified`.
///
pub fn try_set_env_with_name(name: &str) -> Result<()> {
	GitVersion::new()
		.env_var_name(name.to_string())
		.try_set_env()
}

/// Represents the three different formats of a git version.
///
/// If the enclosing repository does not have an appropriated tag,
/// the specified format is ignored and only the commit hash is outputted.
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum GitVersionFormat {
	/// Take only the tag string of the closest git tag omitting the
	/// commit hash.
	///
	/// `TagOnly` is implemented with the special meaning of `--abbrev=0` option
	/// of `git describe`.
	///
	TagOnly,
	
	/// The default output format of `git describe`,
	/// which takes only the tag if the commit of `HEAD` is directly tagged
	/// or the `Long` format otherwise.
	///
	/// `Fancy` is the default format of `git describe`.
	///
	Fancy,
	
	/// Uses always the long format:
	/// ```ignore
	/// format!("{}-{}-g{}", tag_name, commits_ahead, commit_hash)
	/// ```
	///
	/// `Long` is analogous to the `--long` option of `git describe`.
	///
	Long,
}


/// A git version builder, providing fine-grain control over the resulting
/// version name.
///
/// A minimal usage version is `GitVersion::new().set_env()`
///
#[derive(Clone,Debug,Hash,PartialEq,Eq)]
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
	lightweight_tags: bool,
	
	/// Reverse the order of searching tags.
	contains: bool,
	
	/// Travers only the first parent at merge commits.
	first_parent: bool,
	
	/// Allow all kinds of reference as tag
	all_refs: bool,
}

impl GitVersion {
	/// Constructs a new `GitVersion` for setting an environmet variable to a
	/// description of version of the enclosing git repository.
	///
	/// `new` returns an initialized `GitVersion`.
	/// The initialization has the same effect as the following pseudo code:
	/// ```
	/// # use git_version::GitVersion;
	/// # use git_version::GitVersionFormat;
	/// let mut gv: GitVersion;
	/// // ...
	/// # gv = GitVersion::new();
	/// gv.env_var_name("VERSION".to_string())
	///   .dirty_suffix(Some("-modified".to_string()))
	///   .broken_suffix(Some("-broken".to_string()))
	///   .hash_length(0)
	///   .candidates(10)
	///   .format(GitVersionFormat::Fancy)
	///   .lightweight_tags(false)
	///   .contains(false)
	///   .first_parent(false)
	///   .all_refs(false)
	/// # ;assert_eq!(gv, GitVersion::new());
	/// ```
	/// Hence, the default configuration would set the environment variable name
	/// `VERSION`, adds the suffix `-modified` or `-broken` if the working tree
	/// differs from HEAD or can not be evaluated, respectively,
	/// uses the default hash length, candidate number and format,
	/// searches backwards (chronologically) for tags, uses default merge
	/// traversal strategy and does not consider any other refs than
	/// annotated tags.
	///
	pub fn new() -> Self {
		GitVersion {
			env_var_name: "VERSION".to_string(),
			dirty_suffix: Some("-modified".to_string()),
			broken_suffix: Some("-broken".to_string()),
			hash_length: 0,
			candidates: 10,
			format: GitVersionFormat::Fancy,
			lightweight_tags: false,
			contains: false,
			first_parent: false,
			all_refs: false,
		}
	}
	
	/// Defines the name of the environment variable to be set.
	///
	/// The default is `"VERSION"`.
	///
	pub fn env_var_name(&mut self, name: String) -> &mut Self {
		self.env_var_name = name;
		
		self
	}
	
	/// Defines a version suffix if the working tree is dirty.
	///
	/// If `suffix` is set to `Some(str)`, `str` is appended to the git version,
	/// when the working tree differs from HEAD.
	/// If set to `None`, no suffix is generated for a dirty working tree.
	///
	/// `dirty_suffix` is analogous to the `--dirty` option of `git describe`.
	///
	pub fn dirty_suffix(&mut self, suffix: Option<String>) -> &mut Self {
		self.dirty_suffix = suffix;
		
		self
	}
	
	/// Defines a version suffix if the working tree is invalid.
	///
	/// If `suffix` is set to `Some(str)`, `str` is appended to the git version,
	/// when the state of the working tree is invalid or otherwise
	/// indeterminable.
	///
	/// `broken_suffix` implies `dirty_suffix`.
	/// This mean if `dirty_suffix` is `None` while
	/// `broken_suffix` is `Some(_)`,
	/// then `dirty_suffix` is implicitly set to `Some("-dirty".to_string())`.
	///
	/// `broken_suffix` is analogous to the `--broken` option of `git describe`.
	///
	pub fn broken_suffix(&mut self, suffix: Option<String>) -> &mut Self {
		self.broken_suffix = suffix;
		
		self
	}
	
	/// Defines the number of characters in the commit hash, if any.
	///
	/// If `len` is `0`, the default hash length is used.
	///
	/// `hash_length` is analogous to the `--abbrev` option of `git describe`,
	/// but differs in special cases, see next section.
	///
	/// # Compatibility
	///
	/// `hash_length` is analogous to `git describe --abbrev`,
	/// but `--abbrev` has a special meaning if set to `0`,
	/// that causes to omit the trailing commit hash.
	/// This special behavior is recreated in this crate with
	/// [`format`][GitVersion::format()] (see [GitVersionFormat]),
	/// which maps this behavior explicitly
	/// ([`TagOnly`][GitVersionFormat::TagOnly]) using
	/// `git describe --abbrev=0`.
	/// Instead, if `hash_length` is set to `0`,
	/// the default length of commit hashes is used.
	///
	pub fn hash_length(&mut self, len: usize) -> &mut Self {
		self.hash_length = len;
		
		self
	}
	
	/// Defines the number of tags to consider when searching for a tag to
	/// describe HEAD.
	///
	/// If `number` is `0`, HEAD must point to a commit, which is directly
	/// tagged.
	/// This could be useful when using
	/// [`try_set_env`][GitVersion::try_set_env()],
	/// which returns `Err(_)` if `number` is `0` and HEAD is not tagged.
	/// This could be further used to implement special handling in `build.rs`.
	///
	/// `candidates` is analogous to the `--candidates` option of
	/// `git describe`.
	///
	pub fn candidates(&mut self, number: usize) -> &mut Self {
		self.candidates = number;
		
		self
	}
	
	/// Defines the output format of the description.
	///
	/// `format` is realized either with the `--abbrev=0` option,
	/// the `--long` option or non option (default behavior) of `git describe`.
	///
	pub fn format(&mut self, fmt: GitVersionFormat) -> &mut Self {
		self.format = fmt;
		
		self
	}
	
	/// Defines whether to include non-annotated tags.
	///
	/// `lightweight_tags` is analogous to the `--tags` option of
	/// `git describe`.
	///
	pub fn lightweight_tags(&mut self, val: bool) -> &mut Self {
		self.lightweight_tags = val;
		
		self
	}
	
	/// Defines whether to reverse the order of searching tags.
	///
	/// The default (`val == false`) is to search the history for chronological
	/// preceding tags.
	/// This means if HEAD points to an arbitrary commit, it is described by
	/// a preceding tag.
	///
	/// If `val` is `true`, the history is search for chronological succeeding
	/// tags. This is essentially useful in combination with
	/// [`all_refs`][GitVersion::all_refs()], where branches are also taken in
	/// account. In this case the ref (e.g., branch) _containing_ the
	/// HEAD is used to describe it.
	///
	/// `contains` is analogous to the `--contains` option of `git describe`.
	///
	pub fn contains(&mut self, val: bool) -> &mut Self {
		self.contains = val;
		
		self
	}
	
	/// Defines whether to traverse only the first parent of merge commits.
	///
	/// `first_parent` is analogous to the `--first-parent` option of
	/// `git describe`.
	///
	pub fn first_parent(&mut self, val: bool) -> &mut Self {
		self.first_parent = val;
		
		self
	}
	
	/// Defines whether to include all kinds of reference or only tags to
	/// describe a git version.
	///
	/// `all_refs` is analogous to the `--all` option of `git describe`.
	///
	pub fn all_refs(&mut self, val: bool) -> &mut Self {
		self.all_refs = val;
		
		self
	}
	
	/// Use the configuration of `self` to get the git version and set the environment
	/// variable falling back to `"undetermined"` on error.
	///
	/// For more details see
	/// [`set_env_with_default`][GitVersion::set_env_with_default()].
	///
	pub fn set_env(&self) {
		self.set_env_with_default("undetermined");
	}
	
	/// Use the configuration of `self` to get the git version and set the environment
	/// variable falling back to `default_tag` on error.
	///
	/// `set_env_with_default` retrieves the git version using the configuration
	/// defined in `self` and instructs cargo to set the configured environment
	/// variable to the result.
	/// If an error occurred during the determination of the git
	/// version, the environment variable is set to `default_tag` instead.
	///
	/// For further information see
	/// [`try_set_env`][GitVersion::try_set_env()].
	///
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
	/// `try_set_env` tries to retrieve the git version using the configuration
	/// defined in `self` and instructs cargo to set the configured environment
	/// variable to the result.
	/// If an error occurred during the determination of the git version,
	/// an appropriate `Err(_)` is returned and no instructions are passed to
	/// cargo.
	///
	/// This method is intended to be used to do special handling in the
	/// `build.rs` or at runtime, since no environment variable is set on error.
	/// If it is desirable that the environment variable is set anyway,
	/// than [`set_env`][GitVersion::set_env()] or
	/// [`set_env_with_default`][GitVersion::set_env_with_default()]
	/// are more useful.
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
		if self.lightweight_tags {
			cmd.arg("--tags");
		}
		if self.contains {
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




