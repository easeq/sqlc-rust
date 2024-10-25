extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Type;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

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

#[cfg(feature = "with-deadpool")]
#[proc_macro_attribute]
pub fn batch_param(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let ItemStruct {
        attrs, vis, ident, ..
    } = input_struct;
    let args_parsed: Vec<syn::Type> =
        parse_macro_input!(args with Punctuated<Type, syn::token::Comma>::parse_terminated)
            .into_iter()
            .collect();
    if args_parsed.len() != 2 {
        panic!("sqlc_core::batch_param expects exactly two argument");
    }

    let batch_param_type = &args_parsed[0];
    let batch_result_type = &args_parsed[1];

    let fut_ident = format_ident!("{}Fut", ident);
    let fn_ident = format_ident!("{}Fn", ident);

    let type_aliases = quote! {
        type #fut_ident =
            std::pin::Pin<Box<dyn futures::Future<Output = Option<#batch_result_type>> + Send + 'static>>;
        type #fn_ident = Box<
            dyn Fn(
                    deadpool_postgres::Pool,
                    tokio_postgres::Statement,
                    #batch_param_type,
                ) -> #fut_ident
                + Send,
        >;
    };

    let modified_struct = quote! {
        #(#attrs)*
        #vis struct #ident {
            __pool: deadpool_postgres::Pool,
            __stmt: tokio_postgres::Statement,
            __index: usize,
            __items: Vec<#batch_param_type>,
            __fut: #fn_ident,
            __thunk: Option<#fut_ident>,
        }
    }
    .into();

    let modified_struct = parse_macro_input!(modified_struct as ItemStruct);

    let expanded = quote! {
        #type_aliases
        #modified_struct

        impl #ident {
            pub(crate) fn new(
                __pool: deadpool_postgres::Pool,
                __items: Vec<#batch_param_type>,
                __stmt: tokio_postgres::Statement,
                __fut: #fn_ident,
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

            fn set_thunk(mut self: std::pin::Pin<&mut Self>, thunk: #fut_ident) {
                self.__thunk = Some(Box::pin(thunk))
            }
            fn thunk(
                mut self: std::pin::Pin<&mut Self>,
                arg: Self::Param,
                stmt: tokio_postgres::Statement,
                pool: deadpool_postgres::Pool,
            ) -> #fut_ident {
                self.__thunk
                    .take()
                    .unwrap_or_else(move || (self.__fut)(pool.clone(), stmt.clone(), arg.clone()))
            }
        }
        impl futures::Stream for #ident {
            type Item = #batch_result_type;
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
