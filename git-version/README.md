# git-version

Embed git information in your code at compile-time.

```rust
use git_version::git_version;
const GIT_VERSION: &str = git_version!();
```

The version number will have a `-modified` suffix if your git worktree had
untracked or changed files.

These macros do not depend on libgit, but simply uses the `git` binary directly.
So you must have `git` installed somewhere in your `PATH`.

You can also get the version information for all submodules:
```rust
use git_version::git_submodule_versions;
const GIT_SUBMODULE_VERSIONS: &[(&str, &str)] = &git_submodule_versions!();

for (path, version) in GIT_SUBMODULE_VERSIONS {
    println!("{path}: {version}");
}
```

License: BSD-2-Clause
