extern crate proc_macro;
use crate::canonicalize_path;
use crate::git_dependencies;
use crate::utils::run_git;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use syn::{
	bracketed,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	token::{Comma, Eq},
	Expr, Ident, LitStr,
};

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
	fallback: Option<Expr>,
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

pub(crate) fn git_module_versions_impl(args: GitModArgs) -> syn::Result<TokenStream2> {
	let modules = match get_modules() {
		Ok(x) => x,
		Err(err) => return Err(error!("{}", err)),
	};

	let mut describe_paths: Vec<(String, String)> = vec![];

	for path in modules.into_iter() {
		let path_obj = Path::new(&path);
		let path_obj = canonicalize_path(path_obj)?;
		describe_paths.push((path, path_obj));
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

	match describe_modules(describe_paths, &git_describe_args, prefix, suffix) {
		Ok(result) => {
			let dependencies = git_dependencies()?;
			let (paths, versions) = result;

			Ok(quote!({
				#dependencies;

				[#((#paths, #versions)),*]

			}))
		}
		Err(_) if args.fallback.is_some() => Ok(args.fallback.to_token_stream()),
		Err(e) => Err(error!("{}", e)),
	}
}

/// Run `git submodule foreach` command to discover submodules in the project.
fn get_modules() -> Result<Vec<String>, String> {
	let mut args: Vec<String> = "submodule foreach --quiet --recursive"
		.to_string()
		.split(' ')
		.map(|x| x.to_string())
		.collect();

	args.push("echo $displaypath".to_string());

	let result = run_git("git submodule", Command::new("git").args(args))?;

	Ok(result.split('\n').map(|x| x.to_string()).collect())
}

/// Run `git describe` for each submodule to get the git version with the specified args.
fn describe_modules<I, S>(
	paths: Vec<(String, String)>,
	describe_args: I,
	prefix: String,
	suffix: String,
) -> Result<(Vec<String>, Vec<String>), String>
where
	I: IntoIterator<Item = S> + Clone,
	S: AsRef<OsStr>,
{
	let mut paths_out: Vec<String> = vec![];
	let mut versions: Vec<String> = vec![];

	for (rel_path, abs_path) in paths.into_iter() {
		let result = run_git(
			"git describe",
			Command::new("git")
				.current_dir(abs_path)
				.arg("describe")
				.args(describe_args.clone()),
		)?;
		paths_out.push(rel_path);
		versions.push(format!("{}{}{}", prefix, result, suffix))
	}

	Ok((paths_out, versions))
}
