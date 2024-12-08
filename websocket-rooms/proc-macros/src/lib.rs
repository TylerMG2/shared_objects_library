use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Field, Fields, Ident, Lit, Type};

mod networked;

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

// TODO: Add support for non-networked fields maybe using #[serde(skip)], for now all fields are networked and private fields
#[proc_macro_derive(Networked, attributes(private, id))]
pub fn derive_networked(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let optional_name = syn::Ident::new(&format!("{}Optional", name), name.span());

    let data = if let Data::Struct(data) = &input.data {
        data
    } else {
        panic!("#[derive(Networked)] can only be used on structs");
    };

    let fields = if let Fields::Named(fields) = &data.fields {
        &fields.named
    } else {
        panic!("#[derive(Networked)] can only be used on structs with named fields");
    };

    let optional_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;
        quote! {
            pub #field_name: Option<<#field_type as Networked>::Optional>
        }
    });

    let update_from_optional_impl = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! {
            self.#field_name.update_from_optional(optional.#field_name);
        }
    });

    let differences_with_impl = fields.iter().map(|f| {
        let field_name = &f.ident;

        quote! {
            if let Some(diff) = self.#field_name.differences_with(&other.#field_name) {
                let optional = optional.get_or_insert_with(Self::Optional::default);
                optional.#field_name = Some(diff);
            }
        }
    });

    let into_optional_impl = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! {
            #field_name: self.#field_name.into_optional()
        }
    });

    // Removing the unwrap is ideal but would require a different approach to handling fields that are not Option
    let from_optional_impl = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;
        quote! {
            #field_name: <#field_type as Networked>::from_optional(optional.#field_name.unwrap())
        }
    });

    let expanded = quote! {
        #[derive(Serialize, Deserialize, Default, Clone, Copy, Debug)]
        pub struct #optional_name {
            #(#optional_fields,)*
        }

        impl websocket_rooms::core::Networked for #name {
            type Optional = #optional_name;

            fn update_from_optional(&mut self, optional: Option<Self::Optional>) {
                if let Some(optional) = optional {
                    #(#update_from_optional_impl)*
                }
            }

            fn differences_with(&self, other: &Self) -> Option<Self::Optional> {
                let mut optional: Option<Self::Optional> = None;
                #(#differences_with_impl)*
                optional
            }

            fn into_optional(&self) -> Option<Self::Optional> {
                Some(Self::Optional {
                    #(#into_optional_impl,)*
                })
            }

            fn from_optional(optional: Self::Optional) -> Self {
                Self {
                    #(#from_optional_impl,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
