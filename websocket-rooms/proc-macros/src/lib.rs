use helpers::{as_array, assert_has_named_fields, assert_is_struct, assert_type, get_field_with_attribute, get_fields_with_attribute, get_option_inner_type};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Lit};

mod networked;
mod helpers;

#[proc_macro_derive(PlayerFields, attributes(name, disconnected))]
pub fn derive_player_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let data = assert_is_struct(&input.data).expect("PlayerFields can only be applied to structs");
    let fields = assert_has_named_fields(&data.fields).expect("PlayerFields can only be applied to structs with named fields");

    let name_field = get_field_with_attribute(&fields, "name").unwrap_or_else(|e| panic!("{}", e));
    let name_field_name = name_field.ident.as_ref().unwrap();
    let name_field_array = as_array(name_field).expect("Field annotated with `#[name]` must be a fixed size array");
    let name_length = if let Expr::Lit(lit) = &name_field_array.len {
        if let Lit::Int(lit) = &lit.lit {
            let length = lit.base10_parse::<usize>().unwrap();
            if length == 0 {
                panic!("Field annotated with `#[name]` must be a fixed size array with a length greater than 0");
            }
            length
        } else {
            panic!("Field annotated with `#[name]` must be a fixed size array with a literal length");
        }
    } else {
        panic!("Field annotated with `#[name]` must be a fixed size array with a literal length");
    };
    
    let disconnected_field = get_field_with_attribute(&fields, "disconnected").unwrap_or_else(|e| panic!("{}", e));
    let disconnected_field_name = disconnected_field.ident.as_ref().unwrap();
    assert_type(disconnected_field, "bool", "Field annotated with `#[disconnected]` must be of type `bool`");

    // Generate methods to get and set the name and disconnected fields
    let expanded = quote! {
        impl websocket_rooms::core::PlayerFields for #name {
            type Name = [u8; #name_length];

            fn name(&self) -> Self::Name {
                self.#name_field_name
            }

            fn set_name(&mut self, name: Self::Name) {
                self.#name_field_name = name;
            }

            fn disconnected(&self) -> bool {
                self.#disconnected_field_name
            }

            fn set_disconnected(&mut self, disconnected: bool) {
                self.#disconnected_field_name = disconnected;
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(RoomFields, attributes(players, host))]
pub fn derive_room_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let data = assert_is_struct(&input.data).expect("RoomFields can only be applied to structs");
    let fields = assert_has_named_fields(&data.fields).expect("RoomFields can only be applied to structs with named fields");

    let host_field = get_field_with_attribute(&fields, "host").unwrap_or_else(|e| panic!("{}", e));
    let host_field_name = host_field.ident.as_ref().unwrap();
    assert_type(host_field, "u8", "Field annotated with `#[host]` must be of type `u8`");

    let players_field = get_field_with_attribute(&fields, "players").unwrap_or_else(|e| panic!("{}", e));
    let players_field_name = players_field.ident.as_ref().unwrap();
    let players_field_array = as_array(players_field).expect("Field annotated with `#[players]` must be a fixed size array");
    let player_array_type = get_option_inner_type(&*players_field_array.elem);

    // Generate the RoomFields implementation
    let expanded = quote! {
        impl websocket_rooms::core::RoomFields for #name {
            type Player = #player_array_type;

            fn host(&self) -> u8 {
                self.#host_field_name
            }

            fn set_host(&mut self, host: u8) {
                self.#host_field_name = host;
            }

            fn players(&self) -> &[Option<Self::Player>] {
                &self.#players_field_name
            }

            fn players_mut(&mut self) -> &mut [Option<Self::Player>] {
                &mut self.#players_field_name
            }
        }
    };

    TokenStream::from(expanded)
}

// TODO: Add support for non-networked fields maybe using #[serde(skip)], for now all fields are networked and private fields
#[proc_macro_derive(Networked, attributes(private, id))]
pub fn derive_networked(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let optional_name = syn::Ident::new(&format!("{}Optional", name), name.span());

    let data = assert_is_struct(&input.data).expect("Networked can only be applied to structs");
    let fields = assert_has_named_fields(&data.fields).expect("Networked can only be applied to structs with named fields");

    let _private_fields = get_fields_with_attribute(&fields, "private");

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

    // Removing the unwrap is ideal but not sure how to handle this yet
    let from_optional_impl = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;
        quote! {
            #field_name: <#field_type as Networked>::from_optional(optional.#field_name.unwrap_or_default())
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
