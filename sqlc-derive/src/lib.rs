extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[cfg(all(feature = "with-postgres", feature = "with-tokio-postgres"))]
compile_error!(
    "with-postgres and with-tokio-postgres are mutually exclusive and cannot be enabled together"
);

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
                #ident_field_name: row.try_get::<&str, #field_type>(#field_name)?
            }
        }),
        _ => unimplemented!(),
    };

    let module;
    if cfg!(feature = "with-postgres") {
        module = format_ident!("postgres");
    } else if cfg!(feature = "with-tokio-postgres") {
        module = format_ident!("tokio_postgres");
    } else if cfg!(feature = "with-deadpool") {
        module = format_ident!("tokio_postgres");
    } else {
        panic!("one of (with-postgres, with-tokio-postgres, with-deadpool) must be enabled");
    }

    let expanded = quote! {
        impl ::sqlc_core::FromPostgresRow for #ident {
            fn from_row(row: &#module::Row) -> Result<Self, #module::Error> {
                Ok(Self {
                    #(#fields),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
