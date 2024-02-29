extern crate proc_macro;

use parse::Asyncness;
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::ItemImpl;

mod expand;
mod parse;
mod thiscrate;

#[proc_macro_attribute]
pub fn engine(attrs: TokenStream, input: TokenStream) -> TokenStream {
	let mut input = parse_macro_input!(input as ItemImpl);
	let attrs = parse_macro_input!(attrs as Asyncness);
	expand::engine(&mut input, &attrs)
		.unwrap_or_else(syn::Error::into_compile_error)
		.into()
}
