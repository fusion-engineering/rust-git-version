use std::path::PathBuf;

use assert2::{assert, let_assert};
use git_version::{git_describe, git_submodule_versions, git_version};

#[test]
fn git_describe_is_right() {
	let output = std::process::Command::new("git")
		.args(["describe", "--always", "--dirty=-modified"])
		.output()
		.expect("failed to execute git")
		.stdout;

	let_assert!(Ok(name) = std::str::from_utf8(&output));
	let name = name.trim();
	assert!(git_version!(args = ["--always", "--dirty=-modified"]) == name);
	assert!(git_describe!("--always", "--dirty=-modified") == name);
	assert!(git_version!(prefix = "[", suffix = "]") == format!("[{}]", name));
	assert!(git_submodule_versions!() == []);
}

#[test]
fn test_in_external_clone() {
	let_assert!(Ok(tempdir) = tempfile::tempdir());
	let_assert!(Some(project_dir) = std::env::var_os("CARGO_MANIFEST_DIR"));
	let_assert!(Ok(project_dir) = PathBuf::from(project_dir).canonicalize());
	let_assert!(Ok(target_dir) = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).canonicalize());
	let target_dir = target_dir.join("tests_target");

	let_assert!(Ok(result) = std::process::Command::new("git")
		.arg("clone")
		.arg("--quiet")
		.arg("-b")
		.arg("test-root")
		.arg(&project_dir)
		.arg(tempdir.path())
		.status()
	);
	assert!(result.success(), "git clone: {result:?}");

	let_assert!(Ok(result) = std::process::Command::new("git")
		.current_dir(&tempdir)
		.arg("-c")
		.arg("protocol.file.allow=always")
		.arg("submodule")
		.arg("--quiet")
		.arg("update")
		.arg("--init")
		.status()
	);
	assert!(result.success(), "git submodule update --init: {result:?}");

	let_assert!(Ok(result) = std::process::Command::new("cargo")
		.current_dir(&tempdir)
		.arg("add")
		.arg("--path")
		.arg(&project_dir)
		.status()
	);
	assert!(result.success(), "cargo test: {result:?}");

	let_assert!(Ok(result) = std::process::Command::new("cargo")
		.current_dir(&tempdir)
		.arg("test")
		.arg("--target-dir")
		.arg(target_dir)
		.status()
	);
	assert!(result.success(), "cargo test: {result:?}");
}
