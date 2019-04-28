//! Embed git information in your code at compile-time.
//!
//! ```
//! use git_version::git_version;
//! const GIT_VERSION : &str = git_version!();
//! ```
//!
//! The version number will have a `-modified` suffix if your git worktree had
//! untracked or changed files.
//!
//! These macros do not depend on libgit, but simply uses the `git` binary directly.
//! So you must have `git` installed somewhere in your `PATH`.

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use git_version_macro::git_describe;

#[proc_macro_hack]
pub use git_version_macro::git_version;
