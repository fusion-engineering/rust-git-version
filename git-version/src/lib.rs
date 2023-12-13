#![no_std]

//! Embed git information in your code at compile-time.
//!
//! ```
//! use git_version::git_version;
//! const GIT_VERSION: &str = git_version!();
//! ```
//!
//! The version number will have a `-modified` suffix if your git worktree had
//! untracked or changed files.
//!
//! These macros do not depend on libgit, but simply uses the `git` binary directly.
//! So you must have `git` installed somewhere in your `PATH`.
//!
//! You can also get the version information for all submodules:
//! ```
//! use git_version::git_submodule_versions;
//! const GIT_SUBMODULE_VERSIONS: &[(&str, &str)] = &git_submodule_versions!();
//!
//! for (path, version) in GIT_SUBMODULE_VERSIONS {
//!     println!("{path}: {version}");
//! }
//! ```

pub use git_version_macro::{git_submodule_versions, git_version};

/// Run `git describe` at compile time with custom flags.
///
/// This is just a short-hand for `git_version!(args = [...])`,
/// to be backwards compatible with earlier versions of this crate.
#[macro_export]
macro_rules! git_describe {
	($($args:tt)*) => {
		$crate::git_version!(args = [$($args)*])
	};
}
