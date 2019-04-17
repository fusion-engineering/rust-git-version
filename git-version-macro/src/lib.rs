extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::parse_macro_input;

use git_version_impl::{describe_cwd, git_dir_cwd, VERSION_ARGS};

macro_rules! error {
	($($args:tt)*) => { syn::Error::new(proc_macro2::Span::call_site(), format!($($args)*)) };
}

/// Create a token stream representing dependencies on the git state.
fn git_dependencies() -> syn::Result<proc_macro2::TokenStream> {
	let git_dir = git_dir_cwd().map_err(|e| error!("failed to determine .git directory: {}", e))?;

	let head = git_dir.join("logs/HEAD").canonicalize().map_err(|e| error!("failed to canonicalize path to .git/logs/HEAD: {}", e))?;
	let head = head.to_str().ok_or_else(|| error!("invalid UTF-8 in path to .git/logs/HEAD"))?;

	Ok(quote!{
		include_bytes!(#head);
	})
}

struct ArgList{
	args : syn::punctuated::Punctuated<syn::LitStr, syn::token::Comma>,
}

impl syn::parse::Parse for ArgList {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		type Inner = syn::punctuated::Punctuated<syn::LitStr, syn::token::Comma>;
		Ok(Self{args: Inner::parse_terminated(&input)?})
	}
}

struct Nothing;

impl syn::parse::Parse for Nothing {
	fn parse(_input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Nothing)
	}
}

/// Call `git describe` at compile time with custom flags.
///
/// All arguments to the macro must be string literals, and will be passed directly to `git describe`.
///
/// For example:
/// ```no_compile
/// let version = git_describe!("--always", "--dirty");
/// ```
#[proc_macro_hack]
pub fn git_describe(input: TokenStream) -> TokenStream {
	let args : Vec<_> = parse_macro_input!(input as ArgList).args.iter().map(|x| x.value()).collect();

	let tokens = match git_describe_impl(args) {
		Ok(x) => x,
		Err(e) => e.to_compile_error(),
	};

	TokenStream::from(tokens)
}

/// Get the git version for the source code.
///
/// The version string will be created by calling `git describe --always --dirty=-modified`.
/// Use `git_describe!(...)` if you want to pass different flags to `git describe`.
/// All arguments to the macro must be string literals, and will be passed directly to `git describe`.
///
/// For example:
/// ```no_compile
/// let version = git_version();
/// ```
#[proc_macro_hack]
pub fn git_version(input: TokenStream) -> TokenStream {
	parse_macro_input!(input as Nothing);

	let tokens = match git_describe_impl(&VERSION_ARGS) {
		Ok(x) => x,
		Err(e) => e.to_compile_error(),
	};

	TokenStream::from(tokens)
}

fn git_describe_impl<I, S>(args: I) -> syn::Result<proc_macro2::TokenStream> where
	I: IntoIterator<Item = S>,
	S: AsRef<std::ffi::OsStr>,
{
	let version      = describe_cwd(args).map_err(|e| error!("{}", e))?;
	let dependencies = git_dependencies()?;

	Ok(quote!({
		#dependencies;
		#version
	}))
}
