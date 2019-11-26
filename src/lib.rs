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

use proc_macro_hack::proc_macro_hack;

/// Invoke `git describe` at compile time with custom flags.
///
/// All arguments to the macro must be string literals, and will be passed directly to `git describe`.
///
/// For example:
/// ```no_compile
/// const VERSION: &str = git_describe!("--always", "--dirty");
/// ```
#[proc_macro_hack]
pub use git_version_macro::git_describe;

/// Get the git version for the source code.
///
/// The version string will be created by calling `git describe --always --dirty=-modified`.
/// Use [`git_describe`] if you want to pass different flags to `git describe`.
///
/// For example:
/// ```no_compile
/// const VERSION: &str = git_version!();
/// ```
#[proc_macro_hack]
pub use git_version_macro::git_version;
