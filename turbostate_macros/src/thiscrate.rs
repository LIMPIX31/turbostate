use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro_crate::crate_name;
use proc_macro_crate::FoundCrate;
use quote::quote;
use syn::Ident;

#[derive(Debug)]
pub struct Crate(FoundCrate);

impl Default for Crate {
	fn default() -> Self {
		Self(crate_name("turbostate").unwrap())
	}
}

impl Crate {
	#[allow(dead_code)]
	pub fn new(name: &str) -> Self {
		Self(crate_name(name).unwrap_or_else(|_| panic!("{name} is not present in `Cargo.toml`")))
	}

	pub fn path(&self, input: TokenStream) -> TokenStream {
		match &self.0 {
			FoundCrate::Itself => quote!(crate::#input),
			FoundCrate::Name(name) => {
				let ident = Ident::new(name, Span::call_site());
				quote!(#ident::#input)
			}
		}
	}
}
