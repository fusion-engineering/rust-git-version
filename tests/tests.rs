use git_version::{git_version, git_submodule_versions};
use assert2::assert;

#[test]
fn test_root_version() {
	const GIT_VERSION: &str = git_version!();
	assert!(GIT_VERSION == "test-root-v0.1.0-modified");
}

#[test]
fn test_submodule_versions() {
	const VERSIONS: [(&str, &str); 3] = git_submodule_versions!();
	assert!(VERSIONS.len() == 3);
	let mut versions = VERSIONS.as_slice().to_vec();
	versions.sort();
	assert!(versions[0] == ("submodules/submodule-a", "test-submodule-a-v1"));
	assert!(versions[1] == ("submodules/submodule-b", "test-submodule-b-v1-1-g48dbc21"));
	assert!(versions[2] == ("submodules/submodule-c", "ed9ac33"));
}
