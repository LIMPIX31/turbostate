use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, ReturnType};
use syn::ImplItem;
use syn::ItemImpl;
use syn::parse_quote;
use syn::Pat;
use syn::punctuated::Punctuated;
use syn::Result;
use syn::Token;

use crate::parse::{Asyncness, BranchArm};
use crate::thiscrate::Crate;

macro_rules! thiscrate {
	($thiscrate:expr, $($tt:tt)*) => {
		$thiscrate.path(quote!($($tt)*))
	};
}

struct Branch<'ast> {
	ident: &'ast Ident,
	bindings: Punctuated<&'ast Ident, Token![,]>,
	arm: BranchArm,
	asyncness: bool,
}

struct BranchResolver<'ast> {
	pub selfengine: TokenStream,
	pub into_transition: TokenStream,
	pub branches: Vec<Branch<'ast>>,
}

impl<'ast> BranchResolver<'ast> {
	fn visit_impl_item(&mut self, it: &'ast mut ImplItem) -> Result<()> {
		if let ImplItem::Fn(it) = it {
			let mut new_attrs = Vec::with_capacity(it.attrs.len());

			for attr in &mut it.attrs {
				if attr.path().is_ident("branch") {
					let into_transition = &self.into_transition;
					let selfengine = &self.selfengine;

					if let ret @ ReturnType::Default = &mut it.sig.output {
						*ret = parse_quote!(-> impl #into_transition<#selfengine::State, #selfengine::Error>);
					}

					let arm = attr.parse_args()?;

					let branch = Branch {
						bindings: Punctuated::<_, _>::from_iter({
							it.sig.inputs.iter().filter_map(|it| match it {
								FnArg::Typed(typed) => match typed.pat.as_ref() {
									Pat::Ident(id) => Some(&id.ident),
									_ => None,
								},
								_ => None,
							})
						}),
						ident: &it.sig.ident,
						arm,
						asyncness: it.sig.asyncness.is_some(),
					};

					self.branches.push(branch);
				} else {
					new_attrs.push(attr.clone());
				}
			}

			it.attrs = new_attrs;
		}

		Ok(())
	}

	fn visit_item_impl(&mut self, input: &'ast mut ItemImpl) -> Result<()> {
		for item in &mut input.items {
			self.visit_impl_item(item)?;
		}

		Ok(())
	}
}

fn branches<'ast>(input: &'ast mut ItemImpl, engine: &TokenStream, into_transition: &TokenStream) -> Result<BranchResolver<'ast>> {
	let mut this = BranchResolver {
		branches: Default::default(),
		selfengine: quote!(<Self as #engine>),
		into_transition: into_transition.clone(),
	};
	this.visit_item_impl(input)?;
	Ok(this)
}

pub fn engine(input: &mut ItemImpl, attrs: &Asyncness) -> Result<TokenStream> {
	let thiscrate = Crate::default();

	let engine = if attrs.asyncness.is_some() {
		thiscrate!(thiscrate, AsyncEngine)
	} else {
		thiscrate!(thiscrate, Engine)
	};

	let selfengine = quote!(<Self as #engine>);
	let into_transition = thiscrate!(thiscrate, IntoTransition);

	let found_branches = branches(input, &engine, &into_transition)?;

	let branch_arms: Vec<_> = {
		found_branches.branches
			.into_iter()
			.map(|it| {
				let fn_ident = &it.ident;
				let arm = &it.arm;
				let bindings = &it.bindings;

				let optional_dot_await = if it.asyncness {
					quote!(.await)
				} else {
					quote!()
				};

				quote! {
					#arm => #into_transition::<#selfengine::State, #selfengine::Error>::into_transition(self.#fn_ident(#bindings)#optional_dot_await)?,
				}
			})
			.collect()
	};

	let (impl_generics, _, where_clause) = &input.generics.split_for_impl();
	let self_ty = &input.self_ty;
	let items = &input.items;

	let ngx = quote!(<self::#self_ty as #engine>);

	let optional_asyncness = if let Some(asy) = attrs.asyncness {
		quote!(#asy)
	} else {
		quote!()
	};

	Ok(quote! {
		const _: () = {
			impl #impl_generics self::#self_ty #where_clause {
				#(
					#[allow(clippy::wrong_self_convention)]
					#items
				)*
			}

			impl #impl_generics #into_transition<#ngx::State, #ngx::Error> for self::State #where_clause {
				fn into_transition(self) -> ::std::result::Result<#ngx::State, #ngx::Error> {
					Result::Ok(self)
				}
			}

			impl #impl_generics #engine for self::#self_ty #where_clause {
				type State = self::State;
				type Event = self::Event;
				type Error = self::Error;

				#optional_asyncness fn next(&mut self, mut state: #selfengine::State, mut event: #selfengine::Event) -> ::std::result::Result<#selfengine::State, #selfengine::Error> {
					let mut flow = match (state, event) {
						#(#branch_arms)*
					};

					Ok(flow)
				}
			}
		};
	})
}
