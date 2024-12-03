use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Lit, Type};

#[proc_macro_derive(PlayerFields, attributes(name, disconnected))]
pub fn derive_player_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut name_field = None;
    let mut disconnected_field = None;
    let mut name_length = None;

    // Assert its a struct and find the fields annotated with `#[name]` and `#[disconnected]` and make sure they are the correct type
    // Not sure if theres a cleaner way to do this type checking
    if let syn::Data::Struct(ref data) = input.data {
        for field in &data.fields {
            if let Some(_attr) = field.attrs.iter().find(|a| a.path().is_ident("name")) {
                if let Type::Array(array) = &field.ty {
                    if let Type::Path(path) = &*array.elem { // I don't really understand Path but I know elem returns the type of the array which is a Box<Type> hence the *
                        if path.path.is_ident("u8") {
                            if let Expr::Lit(lit) = &array.len { // If the length is a literal
                                if let Lit::Int(lit_int) = &lit.lit { // If that literal is an integer
                                    name_length = Some(lit_int.base10_parse::<usize>().unwrap());
                                }
                            }
                        } else {
                            panic!("Field annotated with `#[name]` must be an array of u8");
                        }
                    } else {
                        panic!("Field annotated with `#[name]` must be an array of u8");
                    }
                } else {
                    panic!("Field annotated with `#[name]` must be an array of u8");
                }
                name_field = Some(field.ident.clone());
            }

            if let Some(_attr) = field.attrs.iter().find(|a| a.path().is_ident("disconnected")) {
                if let Type::Path(path) = &field.ty {
                    if path.path.is_ident("bool") {
                    } else {
                        panic!("Field annotated with `#[disconnected]` must be a bool");
                    }
                } else {
                    panic!("Field annotated with `#[disconnected]` must be a bool");
                }
                disconnected_field = Some(field.ident.clone());
            }
        }
    } else {
        panic!("player_fields can only be applied to structs");
    }

    // Ensure both fields were found
    let name_field = name_field.expect("Missing `#[name]` field");
    let disconnected_field = disconnected_field.expect("Missing `#[disconnected]` field");

    // Generate methods to get and set the name and disconnected fields
    let name_length = name_length.unwrap();
    let expanded = quote! {
        impl PlayerFields for #name {
            fn name(&self) -> &[u8] {
                &self.#name_field
            }

            fn set_name(&mut self, name: &[u8]) {
                let mut new_name = [0u8; #name_length];
                let len = name.len().min(new_name.len());
                new_name[..len].copy_from_slice(&name[..len]); 
                self.#name_field = new_name;
            }

            fn disconnected(&self) -> bool {
                self.#disconnected_field
            }

            fn set_disconnected(&mut self, disconnected: bool) {
                self.#disconnected_field = disconnected;
            }
        }
    };

    TokenStream::from(expanded)
}