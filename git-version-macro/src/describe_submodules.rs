use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::path::Path;

use crate::Args;

macro_rules! error {
	($($args:tt)*) => {
		syn::Error::new(Span::call_site(), format!($($args)*))
	};
}

pub fn git_submodule_versions_impl(args: Args) -> syn::Result<TokenStream2> {
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

	let versions = describe_submodules(&git_dir.join(".."), &modules, &args)
		.map_err(|e| error!("{}", e))?;

	Ok(quote!({
		[#((#modules, #versions)),*]
	}))
}

/// Run `git describe` for each submodule to get the git version with the specified args.
fn describe_submodules(
	root: &Path,
	submodules: &[String],
	args: &Args,
) -> Result<Vec<TokenStream2>, String> {
	let mut versions = Vec::new();

	let git_args = args.git_args.as_ref().map_or_else(
		|| vec!["--always".to_string(), "--dirty=-modified".to_string()],
		|list| list.iter().map(|x| x.value()).collect(),
	);

	for submodule in submodules {
		let path = root.join(submodule);
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
					return Err(e)
				}
			},
		};
		versions.push(version);
	}

	Ok(versions)
}
