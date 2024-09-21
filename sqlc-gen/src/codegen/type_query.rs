use super::type_struct::TypeStruct;
use super::{get_ident, PgDataType};
use convert_case::{Case, Casing};
use core::panic;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, EnumString)]
pub enum QueryCommand {
    #[strum(serialize = ":one")]
    One,
    #[strum(serialize = ":many")]
    Many,
    #[strum(serialize = ":exec")]
    Exec,
    #[strum(serialize = ":execresult")]
    ExecResult,
    #[strum(serialize = ":execrows")]
    ExecRows,
    #[strum(serialize = ":execlastid")]
    ExecLastId,
}

impl QueryCommand {
    pub fn has_return_value(&self) -> bool {
        match *self {
            Self::One | Self::Many => true,
            _ => false,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct QueryValue {
    name: String,
    typ: Option<PgDataType>,
    type_struct: Option<TypeStruct>,
}

impl QueryValue {
    pub fn new<S: Into<String>>(
        name: S,
        typ: Option<PgDataType>,
        type_struct: Option<TypeStruct>,
    ) -> Self {
        Self {
            name: name.into(),
            typ,
            type_struct,
        }
    }

    pub fn get_type(&self) -> String {
        if let Some(typ) = &self.typ {
            typ.to_string()
        } else if let Some(type_struct) = self.type_struct.clone() {
            type_struct.name()
        } else {
            panic!("QueryValue neither has `typ` specified nor `type_struct`");
        }
    }

    pub fn generate_fields_list(&self) -> TokenStream {
        let mut fields_list = quote! {};
        if self.typ.is_some() {
            let ident_name = get_ident(&self.name.clone());
            fields_list = quote! { &#ident_name };
        } else if let Some(type_struct) = self.type_struct.clone() {
            let ident_name = get_ident(&self.name.clone());
            let fields = type_struct
                .fields
                .clone()
                .into_iter()
                .map(|field| {
                    let ident_field_name = get_ident(&field.name());
                    quote! { &#ident_name.#ident_field_name }
                })
                .collect::<Vec<_>>();
            fields_list = quote! { #(#fields),* }
        } else {
            // add panic if necessary
            // panic!("QueryValue neither has `typ` specified nor `type_struct`");
        }

        fields_list
    }
}

impl ToTokens for QueryValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.name.is_empty() {
            let ident_type = get_ident(&self.get_type());
            let ident_name = get_ident(&self.name);
            tokens.extend(quote! {
                #ident_name: #ident_type
            });
        } else if self.typ.is_some() || self.type_struct.is_some() {
            let ident_type = get_ident(&self.get_type());
            tokens.extend(quote! {
                #ident_type
            });
        } else {
            tokens.extend(quote! {});
        }
    }
}

#[derive(Default)]
pub struct TypeQuery {
    name: String,
    cmd: String,
    arg: Option<QueryValue>,
    ret: Option<QueryValue>,
}

impl TypeQuery {
    pub fn new<S: Into<String>>(
        name: S,
        cmd: S,
        arg: Option<QueryValue>,
        ret: Option<QueryValue>,
    ) -> Self {
        Self {
            name: name.into(),
            cmd: cmd.into(),
            arg,
            ret,
        }
    }

    fn constant_name(&self) -> String {
        self.name.to_case(Case::ScreamingSnake)
    }

    pub fn name(&self) -> String {
        self.name.to_case(Case::Snake)
    }

    fn command(&self) -> QueryCommand {
        QueryCommand::from_str(&self.cmd).unwrap()
    }

    fn generate_code(&self) -> TokenStream {
        let ident_name = get_ident(&self.name());
        let ident_const_name = get_ident(&self.constant_name());
        let fields_list = self.arg.clone().unwrap_or_default().generate_fields_list();
        let command = self.command();
        let arg = self.arg.clone().unwrap_or_default();

        let query_method = match command {
            QueryCommand::One => {
                let ret = self.ret.clone().unwrap_or_default();
                quote! {
                    pub(crate) fn #ident_name(&mut self, #arg) -> anyhow::Result<#ret> {
                        let row = self.client.query_one(#ident_const_name, &[#fields_list])?;
                        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
                    }
                }
            }
            QueryCommand::Many => {
                let ret = self.ret.clone().unwrap_or_default();
                quote! {
                    pub(crate) fn #ident_name(&mut self, #arg) -> anyhow::Result<Vec<#ret>> {
                        let rows = self.client.query(#ident_const_name, &[#fields_list])?;
                        let mut result: Vec<#ret> = vec![];
                        for row in rows {
                            result.push(sqlc_core::FromPostgresRow::from_row(&row)?);
                        }

                        Ok(result)
                    }
                }
            }
            QueryCommand::Exec
            | QueryCommand::ExecRows
            | QueryCommand::ExecResult
            | QueryCommand::ExecLastId => quote! {
                pub(crate) fn #ident_name(&mut self, #arg) -> anyhow::Result<()> {
                    self.client.execute(#ident_const_name, &[#fields_list])?;
                    Ok(())
                }
            },
            // _ => quote! {},
        };

        quote! { #query_method }
    }
}

impl ToTokens for TypeQuery {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code().to_token_stream());
    }
}
