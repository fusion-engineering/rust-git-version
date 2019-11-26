extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::parse_macro_input;

mod utils;
use self::utils::{describe_cwd, git_dir_cwd, VERSION_ARGS};

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

struct ArgList {
	args: syn::punctuated::Punctuated<syn::LitStr, syn::token::Comma>,
}

impl syn::parse::Parse for ArgList {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		type Inner = syn::punctuated::Punctuated<syn::LitStr, syn::token::Comma>;
		Ok(Self {
			args: Inner::parse_terminated(&input)?,
		})
	}
}

#[proc_macro_hack]
pub fn git_describe(input: TokenStream) -> TokenStream {
	let args: Vec<_> = parse_macro_input!(input as ArgList).args.iter().map(|x| x.value()).collect();

	let tokens = match git_describe_impl(args) {
		Ok(x) => x,
		Err(e) => e.to_compile_error(),
	};

	TokenStream::from(tokens)
}

#[proc_macro_hack]
pub fn git_version(input: TokenStream) -> TokenStream {
	parse_macro_input!(input as syn::parse::Nothing);

	let tokens = match git_describe_impl(&VERSION_ARGS) {
		Ok(x) => x,
		Err(e) => e.to_compile_error(),
	};

	TokenStream::from(tokens)
}

fn git_describe_impl<I, S>(args: I) -> syn::Result<TokenStream2>
where
	I: IntoIterator<Item = S>,
	S: AsRef<std::ffi::OsStr>,
{
	let version = describe_cwd(args).map_err(|e| error!("{}", e))?;
	let dependencies = git_dependencies()?;

	Ok(quote!({
		#dependencies;
		#version
	}))
}
