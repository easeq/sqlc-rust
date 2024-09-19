use super::type_const::TypeConst;
use super::type_struct::TypeStruct;
use super::{get_ident, CodePartial};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, EnumString)]
enum QueryCommand {
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

#[derive(Default)]
pub struct TypeMethod {
    name: String,
    query_command: String,
    query_const: TypeConst,
    params_struct: TypeStruct,
    row_struct: TypeStruct,
}

impl TypeMethod {
    pub fn new<S: Into<String>>(
        name: S,
        query_command: S,
        query_const: TypeConst,
        params_struct: TypeStruct,
        row_struct: TypeStruct,
    ) -> Self {
        Self {
            name: name.into(),
            query_command: query_command.into(),
            query_const,
            params_struct,
            row_struct,
        }
    }

    fn name(&self) -> String {
        self.name.to_case(Case::Snake)
    }

    fn query_command(&self) -> QueryCommand {
        QueryCommand::from_str(&self.query_command).unwrap()
    }
}

impl CodePartial for TypeMethod {
    fn of_type(&self) -> String {
        "method".to_string()
    }

    fn generate_code(&self) -> TokenStream {
        let ident_name = get_ident(&self.name());
        let ident_row = get_ident(&self.row_struct.name());

        let ident_const_name = get_ident(&self.query_const.name());
        let fields_list = self.params_struct.generate_fields_list();

        let command = self.query_command();

        let params_arg = self.params_struct.get_as_arg(Some("params"));

        let query_method = match command {
            QueryCommand::One => {
                quote! {
                    pub(crate) fn #ident_name(&mut self, #params_arg) -> anyhow::Result<#ident_row> {
                        let row = self.client.query_one(#ident_const_name, &[#fields_list])?;
                        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
                    }
                }
            }
            QueryCommand::Many => {
                quote! {
                    pub(crate) fn #ident_name(&mut self, #params_arg) -> anyhow::Result<Vec<#ident_row>> {
                        let rows = self.client.query(#ident_const_name, &[#fields_list])?;
                        let mut result: Vec<#ident_row> = vec![];
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
                pub(crate) fn #ident_name(&mut self, #params_arg) -> anyhow::Result<()> {
                    self.client.execute(#ident_const_name, &[#fields_list])?;
                    Ok(())
                }
            },
        };

        quote! { #query_method }
    }
}
