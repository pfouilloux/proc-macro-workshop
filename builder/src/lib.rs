extern crate proc_macro;

use proc_macro::TokenStream;

mod derive;

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    derive::derive(input.into()).into()
}
