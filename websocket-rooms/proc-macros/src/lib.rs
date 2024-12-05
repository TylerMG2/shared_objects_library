use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Field, Fields, Ident, Lit, Type};

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
            if has_attr(&field.attrs, "name") {
                if name_field.is_some() { 
                    panic!("Only one field can be annotated with `#[name]`"); 
                }
                let (_, len) = is_fixed_size_array(&field.ty).expect("Field annotated with `#[name]` must be a fixed size array");
                name_length = Some(len);
                name_field = Some(field.ident.clone());
            }

            if has_attr(&field.attrs, "disconnected") {
                if disconnected_field.is_some() { 
                    panic!("Only one field can be annotated with `#[disconnected]`"); 
                }

                if !is_type(&field.ty, "bool") {
                    panic!("Field annotated with `#[disconnected]` must be of type `bool`");
                }

                disconnected_field = Some(field.ident.clone());
            }
        }
    } else {
        panic!("PlayerFields can only be applied to structs");
    }

    // Ensure both fields were found
    let name_field = name_field.expect("Missing `#[name]` field");
    let disconnected_field = disconnected_field.expect("Missing `#[disconnected]` field");

    // Generate methods to get and set the name and disconnected fields
    let name_length = name_length.unwrap();
    let expanded = quote! {
        impl websocket_rooms::core::PlayerFields for #name {
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

#[proc_macro_derive(RoomFields, attributes(players, host))]
pub fn derive_room_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut host_field = None;
    let mut players_field = None;
    let mut player_array_type = None;

    // Process fields
    if let Data::Struct(ref data) = input.data {
        for field in &data.fields {
            if has_attr(&field.attrs, "host") {
                if host_field.is_some() { 
                    panic!("Only one field can be annotated with `#[host]`"); 
                }

                if !is_type(&field.ty, "u8") {
                    panic!("Field annotated with `#[host]` must be of type `u8`");
                }
                host_field = Some(field.ident.clone());
            }

            if has_attr(&field.attrs, "players") {
                if players_field.is_some() { 
                    panic!("Only one field can be annotated with `#[players]`"); 
                }
                let (ident, _) = is_fixed_size_array(&field.ty).expect("Field annotated with `#[players]` must be a fixed size array");
                player_array_type = Some(ident);
                players_field = Some(field.ident.clone());
            }
        }
    } else {
        panic!("RoomFields can only be applied to structs");
    }

    // Ensure fields exist
    let host_field = host_field.expect("Missing `#[host]` field");
    let players_field = players_field.expect("Missing `#[players]` field");
    let player_array_type = player_array_type.expect("Failed to determine player array type");

    // Generate the RoomFields implementation
    let expanded = quote! {
        impl websocket_rooms::core::RoomFields for #name {
            fn host(&self) -> u8 {
                self.#host_field
            }

            fn set_host(&mut self, host: u8) {
                self.#host_field = host;
            }

            fn players(&self) -> &[#player_array_type] {
                &self.#players_field
            }

            fn players_mut(&mut self) -> &mut [#player_array_type] {
                &mut self.#players_field
            }
        }
    };

    TokenStream::from(expanded)
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

fn is_fixed_size_array(ty: &Type) -> Option<(Ident, usize)> {
    if let Type::Array(array) = ty {
        if let Type::Path(path) = &*array.elem {
            // Get type of array
            let ident = path.path.get_ident().unwrap();

            if let Expr::Lit(lit) = &array.len {
                if let Lit::Int(lit_int) = &lit.lit {
                    return Some((ident.clone(), lit_int.base10_parse::<usize>().unwrap()));
                }
            }
        }
    }
    None
}

fn is_type(ty: &Type, name: &str) -> bool {
    if let Type::Path(path) = ty {
        return path.path.is_ident(name);
    }
    false
}

enum FieldVisibility {
    Public,
    Private,
}

enum FieldType {
    Primitive(Type),
    Networked,
    Array(Box<FieldType>),
}

struct FieldInfo {
    ident: Ident,
    visibility: FieldVisibility,
    ty: FieldType,
}

// TODO: Add support for non-networked fields maybe using #[serde(skip)], for now all fields are networked and private fields
#[proc_macro_derive(Networked, attributes(private))]
pub fn derive_networked(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            &fields.named
        } else {
            panic!("Networked can only be applied to structs with named fields");
        }
    } else {
        panic!("Networked can only be applied to structs");
    };

    let networked_fields = fields.iter().map(|field| {
        // Check if the field is a primitive type
        panic!("Not implemented");
        

        if has_attr(&field.attrs, "private") {
            // Make sure its an Option
            if let Type::Path(path) = &field.ty {
                if let Some(ident) = path.path.get_ident() {
                    if ident != "Option" {
                        panic!("Private fields must be of type `Option`");
                    }
                }
                panic!("Private fields must be of type `Option`");
            }
            panic!("Private fields must be of type `Option`");
            FieldVisibility::Private(field.clone())
        } else {
            FieldVisibility::Public(field.clone())
        }
    });

    let expanded = quote! {
        impl websocket_rooms::core::Networked for #name {
            fn serialize(&self) -> Vec<u8> {
                bincode::serialize(self).unwrap()
            }
    
            fn update_from(&mut self, data: &[u8]) {
                *self = bincode::deserialize(data).unwrap();
            }

            fn is_different(&self) -> bool {
                false
            }
        };
    };

    TokenStream::from(expanded)
}