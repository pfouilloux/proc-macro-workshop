use proc_macro2::{TokenStream, Ident};
use syn::{DeriveInput, Data, Field};
use quote::{quote, format_ident};

pub fn derive(input: &DeriveInput) -> TokenStream {
    let caller_name = &input.ident;
    let builder_name = &format_ident!("{}Builder", caller_name.to_string());

    let builder_declaration = write_builder_declaration(builder_name, &input.data);
    let caller_implementation = write_caller_implementation(caller_name,
                                                            builder_name, &input.data);
    let builder_implementation = write_builder_implementation(caller_name,
                                                              builder_name, &input.data);

    quote! {
        #builder_declaration
        #caller_implementation
        #builder_implementation
    }
}

fn write_builder_declaration(name: &Ident, data: &Data) -> TokenStream {
    let fields = write_for_fields(data,
                                  |n, f| write_declaration(n, f));

    quote! {
        struct #name{ #(#fields),* }
    }
}

fn write_caller_implementation(caller_name: &Ident,
                               builder_name: &Ident,
                               data: &Data) -> TokenStream {
    let fields = write_for_fields(data,
                                  |n, _| write_default(n));

    quote! {
        impl #caller_name {
            fn builder() -> #builder_name {
                #builder_name { #(#fields),* }
            }
        }
    }
}

fn write_builder_implementation(caller_name: &Ident,
                                builder_name: &Ident,
                                data: &Data) -> TokenStream {
    let setters = write_for_fields(data,
                                   |n, f| write_setter(n, f));
    let initialisations = write_for_fields(data,
                                           |n, _| write_initialisation(n));

    quote! {
        impl #builder_name {
            #(#setters)*

            pub fn build(&self) -> Result<#caller_name, Box<dyn std::error::Error>> {
                Ok(#caller_name {
                    #(#initialisations),*
                })
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

fn write_declaration(name: &Ident, field: &Field) -> TokenStream {
    let ty = &field.ty;

    quote! { #name: Option<#ty> }
}

fn write_default(name: &Ident) -> TokenStream {
    quote! { #name: None }
}

fn write_setter(name: &Ident, field: &Field) -> TokenStream {
    let ty = &field.ty;

    quote! {
        fn #name(&mut self, #name: #ty) -> &mut Self {
            self.#name = Some(#name);
            self
        }
     }
}

fn write_initialisation(name: &Ident) -> TokenStream {
    let message = format!("Please set {}", name);

    quote! { #name: self.#name.clone().ok_or(Box::<std::error::Error>::from(#message))? }
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

                        impl BobTheBuilder {
                            pub fn build(&self) -> Result<BobThe, Box<dyn std::error::Error>> {
                                Ok(BobThe{})
                            }
                        }
                   }.to_string());
    }

    #[test]
    fn should_derive_with_fields() {
        let input = parse2::<DeriveInput>(
            quote! {
                struct Test {
                    pub a_thing: i32,
                    another_thing: String
                }
            }
        ).unwrap();

        let actual = derive(&input);

        assert_eq!(actual.to_string(),
                   quote! {
                        struct TestBuilder {
                            a_thing: Option<i32>,
                            another_thing: Option<String>
                        }

                        impl Test {
                            fn builder() -> TestBuilder {
                                TestBuilder {
                                    a_thing: None,
                                    another_thing: None
                                }
                            }
                        }

                        impl TestBuilder {
                            fn a_thing(&mut self, a_thing: i32) -> &mut Self {
                                self.a_thing = Some(a_thing);
                                self
                            }

                            fn another_thing(&mut self, another_thing: String) -> &mut Self {
                                self.another_thing = Some(another_thing);
                                self
                            }

                            pub fn build(&self) -> Result<Test, Box<dyn std::error::Error>> {
                                Ok(Test {
                                    a_thing: self.a_thing.clone()
                                        .ok_or(Box::<std::error::Error>::from("Please set a_thing"))?,
                                    another_thing: self.another_thing.clone()
                                        .ok_or(Box::<std::error::Error>::from("Please set another_thing"))?
                                })
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

                        impl TestBuilder {
                            fn field0(&mut self, field0: i32) -> &mut Self {
                                self.field0 = Some(field0);
                                self
                            }

                            fn field1(&mut self, field1: String) -> &mut Self {
                                self.field1 = Some(field1);
                                self
                            }

                            pub fn build(&self) -> Result<Test, Box<dyn std::error::Error>> {
                                Ok(Test {
                                    field0: self.field0.clone()
                                        .ok_or(Box::<std::error::Error>::from("Please set field0"))?,
                                    field1: self.field1.clone()
                                        .ok_or(Box::<std::error::Error>::from("Please set field1"))?
                                })
                            }
                        }
                   }.to_string());
    }
}