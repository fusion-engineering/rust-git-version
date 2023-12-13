use syn::{LitStr, Expr, Ident};
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[derive(Default)]
pub struct Args {
	pub git_args: Option<Punctuated<LitStr, Comma>>,
	pub prefix: Option<Expr>,
	pub suffix: Option<Expr>,
	pub cargo_prefix: Option<Expr>,
	pub cargo_suffix: Option<Expr>,
	pub fallback: Option<Expr>,
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut result = Args::default();
		loop {
			if input.is_empty() {
				break;
			}
			let ident: Ident = input.parse()?;
			let _: syn::token::Eq = input.parse()?;
			let check_dup = |dup: bool| {
				if dup {
					Err(error!("`{} = ` can only appear once", ident))
				} else {
					Ok(())
				}
			};
			match ident.to_string().as_str() {
				"args" => {
					check_dup(result.git_args.is_some())?;
					let content;
					syn::bracketed!(content in input);
					result.git_args = Some(Punctuated::parse_terminated(&content)?);
				}
				"prefix" => {
					check_dup(result.prefix.is_some())?;
					result.prefix = Some(input.parse()?);
				}
				"suffix" => {
					check_dup(result.suffix.is_some())?;
					result.suffix = Some(input.parse()?);
				}
				"cargo_prefix" => {
					check_dup(result.cargo_prefix.is_some())?;
					result.cargo_prefix = Some(input.parse()?);
				}
				"cargo_suffix" => {
					check_dup(result.cargo_suffix.is_some())?;
					result.cargo_suffix = Some(input.parse()?);
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
