use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::Expr;
use syn::Pat;
use syn::Token;

struct BranchPat(Pat);

impl Parse for BranchPat {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self(Pat::parse_single(input)?))
	}
}

pub struct BranchArm {
	pat: Pat,
	guard: Option<(Token![if], Expr)>,
}

impl Parse for BranchArm {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let BranchPat(pat) = input.parse()?;

		let arm = if input.peek(Token![if]) {
			let if_token = input.parse::<Token![if]>()?;
			let expr: Expr = input.parse()?;
			Self {
				pat,
				guard: Some((if_token, expr)),
			}
		} else {
			Self { pat, guard: None }
		};

		Ok(arm)
	}
}

impl ToTokens for BranchArm {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		self.pat.to_tokens(tokens);

		if let Some((if_token, guard)) = &self.guard {
			if_token.to_tokens(tokens);
			guard.to_tokens(tokens);
		}
	}
}

pub struct Asyncness {
	pub asyncness: Option<Token![async]>,
}

impl Parse for Asyncness {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(if input.peek(Token![async]) {
			Self {
				asyncness: Some(input.parse()?),
			}
		} else {
			Self { asyncness: None }
		})
	}
}
