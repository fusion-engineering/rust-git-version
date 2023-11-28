use git_version::{git_describe, git_module_versions, git_version};

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
	let vec = std::process::Command::new("git")
		.args(["submodule", "foreach", "--quiet", "--recursive", "echo $displaypath"])
		.output()
		.expect("failed to execute git")
		.stdout;
	let mut submodules: Vec<String> = String::from_utf8(vec)
		.expect("Failed to gather submodules for test")
		.trim_end()
		.to_string()
		.split("\n")
		.map(|str| str.to_string())
		.collect();

	submodules.retain(|path| path != "");

	let mut expected_result: Vec<(String, String)> = vec![];
	for submodule in submodules.into_iter() {
		let abs_path = std::fs::canonicalize(submodule.clone()).expect("Failed to canonicalize submodule path in test");
		let vec = std::process::Command::new("git")
			.current_dir(abs_path)
			.args(["describe", "--always", "--dirty=-modified"])
			.output()
			.expect("failed to execute git")
			.stdout;
		let name = std::str::from_utf8(&vec[..vec.len() - 1]).expect("non-utf8 error?!");
		expected_result.push((submodule.clone(), name.to_string()))
	}

	let boxed_slice: Box<[(&str, &str)]> = expected_result
		.iter()
		.map(|(path, version)| (path.as_str(), version.as_str()))
		.collect::<Vec<(&str, &str)>>()
		.into_boxed_slice();

	assert_eq!(*boxed_slice, git_module_versions!(args = ["--always", "--dirty=-modified"]));
}
