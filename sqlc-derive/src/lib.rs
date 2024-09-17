extern crate proc_macro;

use postgres::{Error, Row};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

trait FromPostgresRow<'r>: Sized {
    fn from_row(row: &'r Row) -> Result<Self, Error>;
}

#[proc_macro_derive(FromPostgresRow)]
pub fn from_postgres_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let DeriveInput {
        attrs,
        ident,
        vis,
        generics,
        data,
    } = input;

    let fields = match data {
        syn::Data::Struct(data_struct) => data_struct.fields.into_iter().map(|field| {
            let ident_field_name = field.name;
            let field_name = field.name;
            quote! {
                #ident_field_name: row.try_get(#field_name)?
            }
        }),
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl FromPostgresRow for #ident {
            fn from_row(row: &Row) -> Result<Self, Error> {
                Ok(Self {
                    #(#fields)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
