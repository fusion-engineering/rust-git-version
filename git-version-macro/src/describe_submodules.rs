extern crate proc_macro;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use syn::{
	bracketed,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	token::{Comma, Eq},
	Ident, LitStr,
};

use crate::utils::{git_dir, run_git};

macro_rules! error {
	($($args:tt)*) => {
		syn::Error::new(Span::call_site(), format!($($args)*))
	};
}

#[derive(Default)]
pub(crate) struct GitModArgs {
	args: Option<Punctuated<LitStr, Comma>>,
	prefix: Option<LitStr>,
	suffix: Option<LitStr>,
	fallback: Option<LitStr>,
}

impl Parse for GitModArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut result = GitModArgs::default();
		loop {
			if input.is_empty() {
				break;
			}
			let ident: Ident = input.parse()?;
			let _: Eq = input.parse()?;
			let check_dup = |dup: bool| {
				if dup {
					Err(error!("`{} = ` can only appear once", ident))
				} else {
					Ok(())
				}
			};
			match ident.to_string().as_str() {
				"args" => {
					check_dup(result.args.is_some())?;
					let content;
					bracketed!(content in input);
					result.args = Some(Punctuated::parse_terminated(&content)?);
				}
				"prefix" => {
					check_dup(result.prefix.is_some())?;
					result.prefix = Some(input.parse()?);
				}
				"suffix" => {
					check_dup(result.suffix.is_some())?;
					result.suffix = Some(input.parse()?);
				}
				"fallback" => {
					check_dup(result.fallback.is_some())?;
					result.fallback = Some(input.parse()?);
				}
				x => Err(error!("Unexpected argument name `{}`", x))?,
			}
			if input.is_empty() {
				break;
			}
			let _: Comma = input.parse()?;
		}
		Ok(result)
	}
}

pub(crate) fn git_submodule_versions_impl(args: GitModArgs) -> syn::Result<TokenStream2> {
	let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
		.ok_or_else(|| error!("CARGO_MANIFEST_DIR is not set"))?;
	let git_dir = git_dir(&manifest_dir)
		.map_err(|e| error!("failed to determine .git directory: {}", e))?;

	let modules = match get_submodules(&manifest_dir) {
		Ok(x) => x,
		Err(err) => return Err(error!("{}", err)),
	};

	// Ensure that the type of the empty array is still known to the compiler.
	if modules.is_empty() {
		return Ok(quote!([("", ""); 0]));
	}

	let git_describe_args = args.args.map_or_else(
		|| vec!["--always".to_string(), "--dirty=-modified".to_string()],
		|list| list.iter().map(|x| x.value()).collect(),
	);

	let prefix = match args.prefix {
		Some(x) => x.value(),
		_ => "".to_string(),
	};
	let suffix = match args.suffix {
		Some(x) => x.value(),
		_ => "".to_string(),
	};
	let fallback = args.fallback.map(|x| x.value());

	let versions = describe_submodules(&git_dir.join(".."), &modules, &git_describe_args, &prefix, &suffix, fallback.as_deref())
		.map_err(|e| error!("{}", e))?;

	Ok(quote!({
		[#((#modules, #versions)),*]
	}))
}

/// Run `git submodule foreach` command to discover submodules in the project.
fn get_submodules(dir: impl AsRef<Path>) -> Result<Vec<String>, String> {
	let dir = dir.as_ref();
	let result = run_git("git submodule",
		Command::new("git")
			.arg("-C")
			.arg(dir)
			.arg("submodule")
			.arg("foreach")
			.arg("--quiet")
			.arg("--recursive")
			.arg("echo $displaypath"),
	)?;

	Ok(result.lines()
		.filter(|x| !x.is_empty())
		.map(|x| x.to_owned())
		.collect()
	)
}

/// Run `git describe` for each submodule to get the git version with the specified args.
fn describe_submodules<I, S>(
	root: &Path,
	submodules: &[String],
	describe_args: I,
	prefix: &str,
	suffix: &str,
	fallback: Option<&str>,
) -> Result<Vec<String>, String>
where
	I: IntoIterator<Item = S> + Clone,
	S: AsRef<OsStr>,
{
	let mut versions: Vec<String> = vec![];

	for submodule in submodules {
		let path = root.join(submodule);
		// Get the submodule version or fallback.
		let version = match crate::utils::describe(path, describe_args.clone()) {
			Ok(version) => version,
			Err(e) => {
				if let Some(fallback) = fallback {
					fallback.to_owned()
				} else {
					return Err(e)
				}
			},
		};
		versions.push(format!("{}{}{}", prefix, version, suffix))
	}

	Ok(versions)
}
