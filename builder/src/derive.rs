use proc_macro2::{TokenStream, Ident};
use syn::{DeriveInput, Data, Field};
use quote::{quote, format_ident};

pub fn derive(input: &DeriveInput) -> TokenStream {
    let caller_name = &input.ident;
    let builder_name = &format_ident!("{}Builder", caller_name.to_string());

    let builder_struct = write_builder_struct(builder_name, &input.data);
    let caller_impl = write_caller_implementation(caller_name, builder_name, &input.data);

    let astream = quote! {
        #builder_struct
        #caller_impl
    };

    println!("{}", astream.to_string());

    astream
}

fn write_builder_struct(name: &Ident, data: &Data) -> TokenStream {
    let fields = write_for_fields(data,
                                  |n, f| write_field_declaration(n, f));

    quote! {
        struct #name{ #(#fields),* }
    }
}

fn write_caller_implementation(caller_name: &Ident,
                               builder_name: &Ident,
                               data: &Data) -> TokenStream {
    let fields = write_for_fields(data,
                                  |n, _| write_field_default(n));

    quote! {
        impl #caller_name {
            fn builder() -> #builder_name {
                #builder_name { #(#fields),* }
            }
        }
    }
}

fn write_for_fields<T>(data: &Data, transformation: T) -> Vec<TokenStream>
    where T: Fn(&Ident, &Field) -> TokenStream {
    match data {
        Data::Struct(ref data) => data.fields.iter().zip(0..data.fields.len() as u32)
            .map(|(it, index)| transformation(&derive_field_name(it, index), it))
            .collect::<Vec<_>>(),
        _ => unimplemented!(),
    }
}

fn write_field_declaration(name: &Ident, field: &Field) -> TokenStream {
    let ty = &field.ty;

    quote! { #name: Option<#ty> }
}

fn write_field_default(name: &Ident) -> TokenStream {
    quote! { #name: None }
}

fn derive_field_name(field: &Field, _index: u32) -> Ident {
    match &field.ident {
        None => format_ident!("field{}", _index),
        Some(ident) => ident.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::derive::derive;
    use syn::{DeriveInput, parse2};
    use quote::quote;

    #[test]
    fn derive_simple_builder() {
        let input = parse2::<DeriveInput>(
            quote! { struct BobThe(); }
        ).unwrap();

        let actual = derive(&input);

        assert_eq!(actual.to_string(),
                   quote! {
                        struct BobTheBuilder{}

                        impl BobThe {
                            fn builder() -> BobTheBuilder {
                                BobTheBuilder{}
                            }
                        }
                   }.to_string());
    }

    #[test]
    fn should_derive_with_fields() {
        let input = parse2::<DeriveInput>(
            quote! {
                struct Test {
                    pub athing: i32,
                    anotherThing: String
                }
            }
        ).unwrap();

        let actual = derive(&input);

        assert_eq!(actual.to_string(),
                   quote! {
                        struct TestBuilder {
                            athing: Option<i32>,
                            anotherThing: Option<String>
                        }

                        impl Test {
                            fn builder() -> TestBuilder {
                                TestBuilder {
                                    athing: None,
                                    anotherThing: None
                                }
                            }
                        }
                   }.to_string());
    }

    #[test]
    fn should_derive_with_unnamed_fields() {
        let input = parse2::<DeriveInput>(
            quote! {
                struct Test(i32, String);
            }
        ).unwrap();

        let actual = derive(&input);

        assert_eq!(actual.to_string(),
                   quote! {
                        struct TestBuilder {
                            field0: Option<i32>,
                            field1: Option<String>
                        }

                        impl Test {
                            fn builder() -> TestBuilder {
                                TestBuilder {
                                    field0: None,
                                    field1: None
                                }
                            }
                        }
                   }.to_string());
    }
}