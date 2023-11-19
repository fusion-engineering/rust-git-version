extern crate proc_macro;
use crate::git_dependencies;
use crate::utils::describe_modules;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::TypeTuple;
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
	describe_args: Option<Punctuated<LitStr, Comma>>,
	foreach_args: Option<Punctuated<LitStr, Comma>>,
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
				"describe_args" => {
					check_dup(result.describe_args.is_some())?;
					let content;
					bracketed!(content in input);
					result.describe_args = Some(Punctuated::parse_terminated(&content)?);
				}
				"foreach_args" => {
					check_dup(result.foreach_args.is_some())?;
					let content;
					bracketed!(content in input);
					result.foreach_args = Some(Punctuated::parse_terminated(&content)?);
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

pub(crate) fn git_version_modules_impl(args: GitModArgs) -> syn::Result<TokenStream2> {
	let git_describe_args = args.describe_args.map_or_else(
		|| vec!["--always".to_string(), "--dirty=-modified".to_string()],
		|list| list.iter().map(|x| x.value()).collect(),
	);

	let mut git_foreach_args = args.foreach_args.map_or_else(
		|| vec!["--quiet".to_string(), "--recursive".to_string()],
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

	let descibe_args = format!("echo $displaypath:`git describe {}`", git_describe_args.join(" "));

	let mut git_args: Vec<String> = vec!["submodule".to_string(), "foreach".to_string()];
	git_args.append(&mut git_foreach_args);
	git_args.push(descibe_args);

	match describe_modules(&git_args, prefix, suffix) {
		Ok(version) => {
			let dependencies = git_dependencies()?;
			let mut version_tokenable: Vec<(&str, &str)> = vec![];
			for line in version.into_iter() {
				version_tokenable.push((line.0.as_str(), line.1.as_str()));
			}
			Ok(quote!({
				#dependencies;
				[#(#version_tokenable),*];
			}))
		}
		Err(_) if args.fallback.is_some() => Ok(args.fallback.to_token_stream()),
		Err(e) => Err(error!("{}", e)),
	}
}
