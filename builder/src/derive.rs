use proc_macro2::TokenStream;
use syn::DeriveInput;
use quote::{quote, format_ident};

pub fn derive(input: DeriveInput) -> TokenStream {
    let caller_name = input.ident;
    let builder_name = format_ident!("{}Builder", caller_name.to_string());

    quote! {
        struct #builder_name();

        impl #caller_name {
            fn builder() -> #builder_name {
                #builder_name()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::derive::derive;
    use syn::{DeriveInput, parse2};
    use proc_macro2::TokenStream;
    use std::str::FromStr;
    use quote::quote;

    #[test]
    fn derive_bob_the_builder() {
        let input = create_input();

        let result = derive(input);
        println!("{}", result);

        assert_eq!(result.to_string(),
                   expected_tokens().to_string())
    }

    fn create_input() -> DeriveInput {
        let token_stream = TokenStream::from_str("struct BobThe();").unwrap();
        parse2::<DeriveInput>(token_stream).unwrap()
    }

    fn expected_tokens() -> TokenStream {
        quote! {
            struct BobTheBuilder();

            impl BobThe {
                fn builder() -> BobTheBuilder {
                    BobTheBuilder()
                }
            }
        }
    }
}