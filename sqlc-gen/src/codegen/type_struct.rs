use super::get_ident;
use super::PgDataType;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub is_array: bool,
    pub not_null: bool,
    pub number: i32,
    pub data_type: PgDataType,
}

impl StructField {
    pub fn new(
        name: String,
        number: i32,
        data_type: PgDataType,
        is_array: bool,
        not_null: bool,
    ) -> Self {
        Self {
            name,
            number,
            data_type,
            is_array,
            not_null,
        }
    }

    pub fn name(&self) -> String {
        if !self.name.is_empty() {
            self.name.to_case(Case::Snake)
        } else {
            // let self.number == 0 {
            //     panic!("field has neither name nor number");
            // };
            //
            format!("_{}", self.number)
        }
    }

    pub fn data_type(&self) -> TokenStream {
        let mut tokens = self.data_type.to_token_stream();
        // let mut tokens = quote! { #data_type };

        if self.is_array {
            let vec_ident = get_ident("Vec");
            tokens = quote! {
                #vec_ident<#tokens>
            };
        }

        if !self.not_null {
            let option_ident = get_ident("Option");
            tokens = quote! {
                #option_ident<#tokens>
            };
        }

        tokens
    }
}

#[derive(Default, Debug, Clone)]
pub enum StructType {
    #[default]
    Params,
    Row,
    Queries,
    Other {
        prefix: String,
        suffix: String,
    },
}

#[derive(Default, Debug, Clone)]
pub struct TypeStruct {
    name: String,
    struct_type: StructType,
    pub fields: Vec<StructField>,
}

impl TypeStruct {
    pub fn new(name: String, struct_type: StructType, fields: Vec<StructField>) -> Self {
        Self {
            name,
            struct_type,
            fields,
        }
    }

    pub fn name(&self) -> String {
        let name = match &self.struct_type {
            StructType::Params => format!("{}Params", self.name),
            StructType::Row => format!("{}Row", self.name),
            StructType::Queries => self.name.clone(),
            StructType::Other { prefix, suffix } => {
                format!("{}{}{}", prefix, self.name, suffix)
            }
        };

        name.to_case(Case::Pascal)
    }

    pub fn generate_fields_list(&self) -> TokenStream {
        if self.fields.len() == 0 {
            quote! {}
        } else {
            let fields = self
                .fields
                .clone()
                .into_iter()
                .map(|field| {
                    let ident_field_name = get_ident(&field.name());
                    quote! { &params.#ident_field_name }
                })
                .collect::<Vec<_>>();

            quote! { #(#fields),* }
        }
    }

    pub fn generate_code(&self) -> TokenStream {
        if self.fields.len() == 0 {
            quote! {}
        } else {
            let ident_struct = get_ident(&self.name());
            let fields = self
                .fields
                .clone()
                .into_iter()
                .map(|field| self.generate_struct_field_code(field))
                .collect::<Vec<_>>();

            quote! {
                #[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
                pub(crate) struct #ident_struct {
                    #(#fields)*
                }
            }
        }
    }

    fn generate_struct_field_code(&self, field: StructField) -> TokenStream {
        let field_name_ident = get_ident(&field.name());
        let field_type_ident = field.data_type();

        quote! {
            pub(crate) #field_name_ident: #field_type_ident,
        }
    }
}
