use git_version::{git_describe, git_version, git_version_modules};

#[test]
fn git_describe_is_right() {
	let vec = std::process::Command::new("git")
		.args(["describe", "--always", "--dirty=-modified"])
		.output()
		.expect("failed to execute git")
		.stdout;
	let name = std::str::from_utf8(&vec[..vec.len() - 1]).expect("non-utf8 error?!");
	println!("name = {}", name);
	println!("GIT_VERSION = {}", git_version!(args = ["--always", "--dirty=-modified"]));
	assert_eq!(git_version!(args = ["--always", "--dirty=-modified"]), name);
	assert_eq!(git_describe!("--always", "--dirty=-modified"), name);
	assert_eq!(git_version!(prefix = "[", suffix = "]"), format!("[{}]", name));
}

#[test]
fn test_modules_macro_gives_expected_output() {
	let module_versions = git_version_modules!(
		prefix = "pre-",
		suffix = "-suff",
		args = ["--always", "--dirty=-modified", "--tags"]
	);
	println!("{module_versions:#?}");
}
