extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::parse_macro_input;

use git_version_impl::{version_cwd, describe_cwd, git_dir_cwd};

/// Create a token stream representing dependencies on the git state.
fn git_dependencies() -> proc_macro2::TokenStream {
	let git_dir = git_dir_cwd().expect("failed to determine .git directory");

	let head = git_dir.join("HEAD");
	let head = head.canonicalize().expect("failed to canonicalize path to .git/HEAD");
	let head = head.to_str().expect("invalid UTF-8 in path to .git/HEAD");
	let mut tokens = quote!{
		include_bytes!(#head);
	};

	for entry in git_dir.join("refs/heads").read_dir().expect("failed to iterate over git heads") {
		let entry = entry.expect("error occurred while iterating over git heads").path();
		let entry = entry.canonicalize().expect("failed to canonicalize path to .git/refs/heads");
		let entry = entry.to_str().expect("invalid UTF-8 in path to head in .git/refs/heads");
		tokens.extend(quote!{
			include_bytes!(#entry);
		});
	}

	tokens
}

#[proc_macro]
pub fn declare(input: TokenStream) -> TokenStream {

	let identifier = proc_macro2::TokenStream::from(input);

	let version = version_cwd().expect("failed to determine git version");
	let mut tokens = quote!{
		const #identifier: &'static str = #version;
	};

	tokens.extend(git_dependencies());
	tokens.into()
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

	let version      = describe_cwd(&args).expect("failed to run `git describe`");
	let dependencies = git_dependencies();

	quote!({
		#dependencies;
		#version
	}).into()
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

	let version      = version_cwd().expect("failed to run `git describe`");
	let dependencies = git_dependencies();

	quote!({
		#dependencies;
		#version
	}).into()
}
