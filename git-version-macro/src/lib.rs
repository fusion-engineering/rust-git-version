extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::{
	bracketed,
	parse::{Parse, ParseStream},
	parse_macro_input,
	punctuated::Punctuated,
	token::{Comma, Eq},
	Ident, LitStr,
};

mod utils;
use self::utils::{describe_cwd, git_dir_cwd};

macro_rules! error {
	($($args:tt)*) => {
		syn::Error::new(Span::call_site(), format!($($args)*))
	};
}

fn canonicalize_path(path: &Path) -> syn::Result<String> {
	Ok(path
		.canonicalize()
		.map_err(|e| error!("failed to canonicalize {}: {}", path.display(), e))?
		.into_os_string()
		.into_string()
		.map_err(|file| error!("invalid UTF-8 in path to {}", PathBuf::from(file).display()))?
	)
}

/// Create a token stream representing dependencies on the git state.
fn git_dependencies() -> syn::Result<TokenStream2> {
	let git_dir = git_dir_cwd().map_err(|e| error!("failed to determine .git directory: {}", e))?;

	let deps: Vec<_> = ["logs/HEAD", "index"].iter().flat_map(|&file| {
		canonicalize_path(&git_dir.join(file)).map(Some).unwrap_or_else(|e|  {
			eprintln!("Failed to add dependency on the git state: {}. Git state changes might not trigger a rebuild.", e);
			None
		})
	}).collect();

	Ok(quote! {
		#( include_bytes!(#deps); )*
	})
}

#[derive(Default)]
struct Args {
	git_args: Option<Punctuated<LitStr, Comma>>,
}

impl Parse for Args {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut result = Args::default();
		loop {
			if input.is_empty() { break; }
			let ident: Ident = input.parse()?;
			let _: Eq = input.parse()?;
			match ident.to_string().as_str() {
				"args" => {
					if result.git_args.is_some() {
						Err(error!("`args = ` can only appear once"))?;
					}
					let content;
					bracketed!(content in input);
					result.git_args = Some(Punctuated::parse_terminated(&content)?);
				}
				x => Err(error!("Unexpected argument name `{}`", x))?,
			}
			if input.is_empty() { break; }
			let _: Comma = input.parse()?;
		}
		Ok(result)
	}
}

#[proc_macro_hack]
pub fn git_version(input: TokenStream) -> TokenStream {
	let args = parse_macro_input!(input as Args);

	let tokens = match git_version_impl(args) {
		Ok(x) => x,
		Err(e) => e.to_compile_error(),
	};

	TokenStream::from(tokens)
}

fn git_version_impl(args: Args) -> syn::Result<TokenStream2> {
	let git_args = args.git_args.map_or_else(
		|| vec!["--always".to_string(), "--dirty=-modified".to_string()],
		|list| list.iter().map(|x| x.value()).collect()
	);

	let version = describe_cwd(&git_args).map_err(|e| error!("{}", e))?;
	let dependencies = git_dependencies()?;

	Ok(quote!({
		#dependencies;
		#version
	}))
}
