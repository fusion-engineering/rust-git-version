extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use std::path::Path;
use syn::parse_macro_input;

mod utils;
use self::utils::{describe_cwd, git_dir_cwd, VERSION_ARGS};

macro_rules! error {
	($($args:tt)*) => { syn::Error::new(Span::call_site(), format!($($args)*)) };
}

/// Canonicalize the path to a file inside the git folder.
fn canonicalize_git_path(git_dir: impl AsRef<Path>, file: impl AsRef<Path>) -> syn::Result<String> {
	let git_dir = git_dir.as_ref();
	let file = file.as_ref();

	let path = git_dir.join(file);
	let path = path
		.canonicalize()
		.map_err(|e| error!("failed to canonicalize {}: {}", path.display(), e))?;
	let path = path.to_str().ok_or_else(|| error!("invalid UTF-8 in path to {}", file.display()))?;

	Ok(String::from(path))
}

/// Create a token stream representing dependencies on the git state.
fn git_dependencies() -> syn::Result<TokenStream2> {
	let git_dir = git_dir_cwd().map_err(|e| error!("failed to determine .git directory: {}", e))?;

	let head = canonicalize_git_path(&git_dir, "logs/HEAD");
	let index = canonicalize_git_path(&git_dir, "index");

	if let Some(error) = head.as_ref().err().or(index.as_ref().err()) {
		eprintln!(
			"Failed to add dependencies on the git state: {}. The crate may not rebuild if the git state changes.",
			error
		)
	}

	let head = head.ok().map(|x| quote! { include_bytes!(#x); });
	let index = index.ok().map(|x| quote! { include_bytes!(#x); });

	Ok(quote! {
		#head
		#index
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

struct Nothing;

impl syn::parse::Parse for Nothing {
	fn parse(_input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Nothing)
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
	parse_macro_input!(input as Nothing);

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
