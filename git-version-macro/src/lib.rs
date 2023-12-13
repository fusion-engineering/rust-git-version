use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};

macro_rules! error {
	($($args:tt)*) => {
		syn::Error::new(proc_macro2::Span::call_site(), format!($($args)*))
	};
}

mod args;
mod utils;

/// Get the git version for the source code.
///
/// The following (named) arguments can be given:
///
/// - `args`: The arguments to call `git describe` with.
///   Default: `args = ["--always", "--dirty=-modified"]`
///
/// - `prefix`, `suffix`:
///   The git version will be prefixed/suffexed by these strings.
///
/// - `cargo_prefix`, `cargo_suffix`:
///   If either is given, Cargo's version (given by the CARGO_PKG_VERSION
///   environment variable) will be used if git fails instead of giving an
///   error. It will be prefixed/suffixed by the given strings.
///
/// - `fallback`:
///   If all else fails, this string will be given instead of reporting an
///   error.
///
/// # Examples
///
/// ```
/// # use git_version::git_version;
/// const VERSION: &str = git_version!();
/// ```
///
/// ```
/// # use git_version::git_version;
/// const VERSION: &str = git_version!(args = ["--abbrev=40", "--always"]);
/// ```
///
/// ```
/// # use git_version::git_version;
/// const VERSION: &str = git_version!(prefix = "git:", cargo_prefix = "cargo:", fallback = "unknown");
/// ```
#[proc_macro]
pub fn git_version(input: TokenStream) -> TokenStream {
	let args = syn::parse_macro_input!(input as args::Args);

	let tokens = match git_version_impl(args) {
		Ok(x) => x,
		Err(e) => e.to_compile_error(),
	};

	TokenStream::from(tokens)
}

fn git_version_impl(args: args::Args) -> syn::Result<TokenStream2> {
	let git_args = args.git_args.map_or_else(
		|| vec!["--always".to_string(), "--dirty=-modified".to_string()],
		|list| list.iter().map(|x| x.value()).collect(),
	);

	let cargo_fallback = args.cargo_prefix.is_some() || args.cargo_suffix.is_some();

	let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
		.ok_or_else(|| error!("CARGO_MANIFEST_DIR is not set"))?;

	match utils::describe(manifest_dir, git_args) {
		Ok(version) => {
			let dependencies = utils::git_dependencies()?;
			let prefix = args.prefix.iter();
			let suffix = args.suffix;
			Ok(quote!({
				#dependencies;
				concat!(#(#prefix,)* #version, #suffix)
			}))
		}
		Err(_) if cargo_fallback => {
			if let Ok(version) = std::env::var("CARGO_PKG_VERSION") {
				let prefix = args.cargo_prefix.iter();
				let suffix = args.cargo_suffix;
				Ok(quote!(concat!(#(#prefix,)* #version, #suffix)))
			} else if let Some(fallback) = args.fallback {
				Ok(fallback.to_token_stream())
			} else {
				Err(error!("Unable to get git or cargo version"))
			}
		}
		Err(_) if args.fallback.is_some() => Ok(args.fallback.to_token_stream()),
		Err(e) => Err(error!("{}", e)),
	}
}

/// Get the git version of all submodules below the cargo project.
///
/// This macro expands to `[(&str, &str), N]` where `N` is the total number of
/// submodules below the root of the project (evaluated recursively)
///
/// Each entry in the array is a tuple of the submodule path and the version information.
///
/// The following (named) arguments can be given:
///
/// - `args`: The arguments to call `git describe` with.
///   Default: `args = ["--always", "--dirty=-modified"]`
///
/// - `prefix`, `suffix`:
///   The git version for each submodule will be prefixed/suffixed
///   by these strings.
///
/// - `fallback`:
///   If all else fails, this string will be given instead of reporting an
///   error. This will yield the same type as if the macro was a success, but
///   format will be `[("relative/path/to/submodule", {fallback})]`
///
/// # Examples
///
/// ```
/// # use git_version::git_submodule_versions;
/// # const N: usize = 0;
/// const MODULE_VERSIONS: [(&str, &str); N] = git_submodule_versions!();
/// for (path, version) in MODULE_VERSIONS {
///     println!("{path}: {version}");
/// }
/// ```
///
/// ```
/// # use git_version::git_submodule_versions;
/// # const N: usize = 0;
/// const MODULE_VERSIONS: [(&str, &str); N] = git_submodule_versions!(args = ["--abbrev=40", "--always"]);
/// ```
///
/// ```
/// # use git_version::git_submodule_versions;
/// # const N: usize = 0;
/// const MODULE_VERSIONS: [(&str, &str); N] = git_submodule_versions!(prefix = "git:", fallback = "unknown");
/// ```
#[proc_macro]
pub fn git_submodule_versions(input: TokenStream) -> TokenStream {
	let args = syn::parse_macro_input!(input as args::Args);

	let tokens = match git_submodule_versions_impl(args) {
		Ok(x) => x,
		Err(e) => e.to_compile_error(),
	};

	TokenStream::from(tokens)
}

fn git_submodule_versions_impl(args: args::Args) -> syn::Result<TokenStream2> {
	if let Some(cargo_prefix) = &args.cargo_prefix {
		return Err(syn::Error::new_spanned(cargo_prefix, "invalid argument `cargo_prefix` for `git_submodule_versions!()`"));
	}
	if let Some(cargo_suffix) = &args.cargo_suffix {
		return Err(syn::Error::new_spanned(cargo_suffix, "invalid argument `cargo_suffix` for `git_submodule_versions!()`"));
	}

	let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
		.ok_or_else(|| error!("CARGO_MANIFEST_DIR is not set"))?;
	let git_dir = crate::utils::git_dir(&manifest_dir)
		.map_err(|e| error!("failed to determine .git directory: {}", e))?;

	let modules = match crate::utils::get_submodules(&manifest_dir) {
		Ok(x) => x,
		Err(err) => return Err(error!("{}", err)),
	};

	// Ensure that the type of the empty array is still known to the compiler.
	if modules.is_empty() {
		return Ok(quote!([("", ""); 0]));
	}

	let git_args = args.git_args.as_ref().map_or_else(
		|| vec!["--always".to_string(), "--dirty=-modified".to_string()],
		|list| list.iter().map(|x| x.value()).collect(),
	);

	let root_dir = git_dir.join("..");
	let mut versions = Vec::new();
	for submodule in &modules {
		let path = root_dir.join(submodule);
		// Get the submodule version or fallback.
		let version = match crate::utils::describe(path, &git_args) {
			Ok(version) => {
				let prefix = args.prefix.iter();
				let suffix = args.suffix.iter();
				quote!{
					::core::concat!(#(#prefix,)* #version #(, #suffix)*)
				}
			}
			Err(e) => {
				if let Some(fallback) = &args.fallback {
					quote!( #fallback )
				} else {
					return Err(error!("{}", e));
				}
			},
		};
		versions.push(version);
	}

	Ok(quote!({
		[#((#modules, #versions)),*]
	}))
}
