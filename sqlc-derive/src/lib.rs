extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[cfg(feature = "with-deadpool")]
const BATCH_PARAM: &str = "batch_param";
#[cfg(feature = "with-deadpool")]
const BATCH_RESULT: &str = "batch_result";

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

    #[cfg(feature = "with-postgres")]
    {
        let expanded = quote! {
            impl sqlc_core::FromPostgresRow for #ident {
                fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
                    Ok(Self {
                        #(#fields),*
                    })
                }
            }
        };

        TokenStream::from(expanded)
    }

    #[cfg(feature = "with-tokio-postgres")]
    {
        let expanded = quote! {
            impl sqlc_core::FromPostgresRow for #ident {
                fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
                    Ok(Self {
                        #(#fields),*
                    })
                }
            }
        };

        TokenStream::from(expanded)
    }
}

fn get_field_type(field: &syn::Field) -> Option<&str> {
    field.attrs.iter().find_map(|attr| {
        if attr.path().is_ident(BATCH_PARAM) {
            Some(BATCH_PARAM)
        } else if attr.path().is_ident(BATCH_RESULT) {
            Some(BATCH_RESULT)
        } else {
            None
        }
    })
}

#[cfg(feature = "with-deadpool")]
#[proc_macro_attribute]
pub fn batch_result_type(args: TokenStream, input: TokenStream) -> TokenStream {
    use quote::format_ident;
    use syn::parse::Parser;
    use syn::ItemStruct;
    use syn::Type;

    let mut input_struct = parse_macro_input!(input as ItemStruct);
    let ident = input_struct.ident.clone();
    let _args = parse_macro_input!(args as syn::parse::Nothing);

    let mut batch_param_type: Option<Type> = None;
    let mut batch_result_type: Option<Type> = None;
    if let syn::Fields::Named(ref mut fields) = input_struct.fields {
        assert!(fields.named.len() == 2, "#[batch_result_type] struct can have only 2 mandatory struct fields marked with #[batch_param] and #[batch_result] respectively");

        for _ in 1..=2 {
            if let Some(pair) = fields.named.pop() {
                let field = pair.value();
                if let Some(field_type) = get_field_type(&field) {
                    if field_type == BATCH_PARAM && batch_param_type.is_none() {
                        batch_param_type = Some(field.ty.clone());
                    } else if field_type == BATCH_RESULT && batch_result_type.is_none() {
                        batch_result_type = Some(field.ty.clone());
                    } else {
                        panic!("unknown field type in struct marked #[batch_result_type]")
                    }
                }
            }
        }

        let injected_fields = vec![
            quote!(__pool: deadpool_postgres::Pool),
            quote!(__stmt: tokio_postgres::Statement),
            quote!(__index: usize),
            quote!(__items: Vec<#batch_param_type>),
            quote!(__fut: sqlc_core::BatchResultsFn<#batch_param_type, #batch_result_type>),
            quote!(__thunk: Option<sqlc_core::BatchResultsFut<#batch_result_type>>),
        ];

        for field in injected_fields {
            fields.named.push(
                syn::Field::parse_named
                    .parse2(field)
                    .expect("could not parse batch_result_type field"),
            );
        }
    } else {
        panic!(
            "missing struct named-fields each marked with #[batch_param] and #[batch_result] respectively"
        )
    }

    let modified_struct = quote!(#input_struct).into();
    let modified_struct = parse_macro_input!(modified_struct as ItemStruct);

    let expanded = quote! {
        #modified_struct

        impl #ident {
            pub(crate) fn new(
                __pool: deadpool_postgres::Pool,
                __items: Vec<#batch_param_type>,
                __stmt: tokio_postgres::Statement,
                __fut: sqlc_core::BatchResultsFn<#batch_param_type, #batch_result_type>,
            ) -> Self {
                Self {
                    __pool,
                    __items,
                    __stmt,
                    __fut,
                    __index: 0,
                    __thunk: None,
                }
            }
        }

        impl sqlc_core::BatchResult for #ident {
            type Param = #batch_param_type;
            fn inc_index(mut self: std::pin::Pin<&mut Self>) {
                self.__index += 1;
            }
            fn stmt(&self) -> tokio_postgres::Statement {
                self.__stmt.clone()
            }
            fn pool(&self) -> deadpool_postgres::Pool {
                self.__pool.clone()
            }
            fn current_item(&self) -> Option<Self::Param> {
                if self.__index < self.__items.len() {
                    Some(self.__items[self.__index].clone())
                } else {
                    None
                }
            }
            fn set_thunk(
                mut self: std::pin::Pin<&mut Self>,
                thunk: sqlc_core::BatchResultsFut<#batch_result_type>,
            ) {
                self.__thunk = Some(Box::pin(thunk))
            }
            fn thunk(
                mut self: std::pin::Pin<&mut Self>,
                arg: Self::Param,
                stmt: tokio_postgres::Statement,
                pool: deadpool_postgres::Pool,
            ) -> sqlc_core::BatchResultsFut<#batch_result_type> {
                self.__thunk
                    .take()
                    .unwrap_or_else(move || (self.__fut)(pool.clone(), stmt.clone(), arg.clone()))
            }
        }

        impl futures::Stream for #ident {
            type Item = Result<#batch_result_type, sqlc_core::Error>;
            fn poll_next(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Self::Item>> {
                <Self as sqlc_core::BatchResult>::poll_next(self, cx)
            }
        }

    };

    // panic!("{:?}", expanded.to_string());

    expanded.into()
}
