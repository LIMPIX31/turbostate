extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::ItemImpl;

mod expand;
mod parse;
mod thiscrate;

#[proc_macro_attribute]
pub fn engine(_attrs: TokenStream, input: TokenStream) -> TokenStream {
	let mut input = parse_macro_input!(input as ItemImpl);
	expand::engine(&mut input)
		.unwrap_or_else(syn::Error::into_compile_error)
		.into()
}
