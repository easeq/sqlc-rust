extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromPostgresRow)]
pub fn from_postgres_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let DeriveInput { ident, data, .. } = input;

    let fields = match data {
        syn::Data::Struct(data_struct) => data_struct.fields.into_iter().map(|field| {
            let ident_field_name = field.ident.clone().unwrap();
            let field_type = field.ty;
            let field_name = field.ident.unwrap().to_string();
            quote! {
                #ident_field_name: row.get::<&str, #field_type>(#field_name)
            }
        }),
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl sqlc_core::FromPostgresRow for #ident {
            fn from_row(row: &Row) -> Result<Self, Error> {
                Ok(Self {
                    #(#fields),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
