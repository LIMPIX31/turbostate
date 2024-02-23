use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::FnArg;
use syn::ImplItem;
use syn::ItemImpl;
use syn::parse_quote;
use syn::Pat;
use syn::punctuated::Punctuated;
use syn::Result;
use syn::Token;

use crate::parse::BranchArm;
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
}

struct BranchResolver<'ast, 'a> {
	pub thiscrate: &'a Crate,
	pub branches: Vec<Branch<'ast>>,
	pub pre_hooks: Vec<Ident>,
	pub post_hooks: Vec<Ident>,
}

impl<'ast, 'a> BranchResolver<'ast, 'a> {
	fn visit_impl_item(&mut self, it: &'ast mut ImplItem) -> Result<()> {
		if let ImplItem::Fn(it) = it {
			let mut new_attrs = Vec::with_capacity(it.attrs.len());

			for attr in &mut it.attrs {
				if attr.path().is_ident("branch") {
					let engine = thiscrate!(self.thiscrate, Engine);
					it.sig.output = parse_quote!(-> <Self as #engine>::Flow);

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
					};

					self.branches.push(branch);
				} else if attr.path().is_ident("post") {
					self.post_hooks.push(it.sig.ident.clone());
				} else if attr.path().is_ident("pre") {
					self.pre_hooks.push(it.sig.ident.clone());
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

fn branches<'ast, 'a>(input: &'ast mut ItemImpl, thiscrate: &'a Crate) -> Result<BranchResolver<'ast, 'a>> {
	let mut this = BranchResolver {
		branches: Default::default(),
		pre_hooks: Vec::new(),
		post_hooks: Vec::new(),
		thiscrate,
	};
	this.visit_item_impl(input)?;
	Ok(this)
}

pub fn engine(input: &mut ItemImpl) -> Result<TokenStream> {
	let thiscrate = Crate::default();

	let engine = thiscrate!(thiscrate, Engine);
	let flow = thiscrate!(thiscrate, Flow);

	let found_branches = branches(input, &thiscrate)?;

	let branch_arms: Vec<_> = {
		found_branches.branches
			.into_iter()
			.map(|it| {
				let fn_ident = &it.ident;
				let arm = &it.arm;
				let bindings = &it.bindings;

				quote! {
					#arm => self.#fn_ident(#bindings).await,
				}
			})
			.collect()
	};

	let pre_hooks = found_branches.pre_hooks;
	let post_hooks = found_branches.post_hooks;

	let generics = &input.generics;
	let self_ty = &input.self_ty;
	let items = &input.items;

	let flow = quote!(#flow<<Self as #engine>::State, <Self as #engine>::Error>);

	Ok(quote! {
		const _: () = {
			impl #generics self::#self_ty #generics {
				#(
					#[allow(clippy::wrong_self_convention)]
					#items
				)*
			}

			impl #generics #engine for self::#self_ty {
				type Flow = #flow;
				type State = self::State;
				type Event = self::Event;
				type Error = self::Error;

				async fn next(&mut self, mut state: <Self as #engine>::State, mut event: <Self as #engine>::Event) -> <Self as #engine>::Flow {
					#(match self.#pre_hooks(&mut state, &mut event).await {
						Err(err) => return <Self as #engine>::Flow::Failure(err),
						_ => (),
					};)*
					let mut flow = match (state, event) {
						#(#branch_arms)*
					};
					#(match self.#post_hooks(&mut flow).await {
						Err(err) => return <Self as #engine>::Flow::Failure(err),
						_ => (),
					};)*
					flow
				}
			}
		};
	})
}
