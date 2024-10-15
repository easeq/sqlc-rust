use super::type_struct::TypeStruct;
use super::{get_ident, DataType, PgDataType};
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

    pub fn get_type(&self) -> DataType {
        if let Some(typ) = &self.typ {
            typ.as_data_type()
        } else if let Some(type_struct) = self.type_struct.clone() {
            type_struct.data_type()
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
            let ident_type = &self.get_type();
            let ident_name = get_ident(&self.name);
            tokens.extend(quote! {
                #ident_name: #ident_type
            });
        } else if self.typ.is_some() || self.type_struct.is_some() {
            let ident_type = &self.get_type();
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
    use_async: bool,
    use_deadpool: bool,
}

impl TypeQuery {
    pub fn new<S: Into<String>>(
        name: S,
        cmd: S,
        arg: Option<QueryValue>,
        ret: Option<QueryValue>,
        use_async: bool,
        use_deadpool: bool,
    ) -> Self {
        Self {
            name: name.into(),
            cmd: cmd.into(),
            arg,
            ret,
            use_async,
            use_deadpool,
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
        let client = if self.use_deadpool {
            quote!(self.client().await)
        } else {
            quote!(self.client)
        };

        let query_method = match command {
            QueryCommand::One => {
                let ret = self.ret.clone().unwrap_or_default();
                let sig = quote! { fn #ident_name(&mut self, #arg) -> anyhow::Result<#ret> };
                let fetch_stmt = quote! {
                    let row = #client.query_one(#ident_const_name, &[#fields_list])
                };
                let fn_body = quote! {
                    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
                };
                QueryMethod::new(sig, fn_body, fetch_stmt, self.use_async)
            }
            QueryCommand::Many => {
                let ret = self.ret.clone().unwrap_or_default();
                let sig = quote! { fn #ident_name(&mut self, #arg) -> anyhow::Result<Vec<#ret>> };
                let fetch_stmt = quote! {
                    let rows = #client.query(#ident_const_name, &[#fields_list])
                };
                let fn_body = quote! {
                    let mut result: Vec<#ret> = vec![];
                    for row in rows {
                        result.push(sqlc_core::FromPostgresRow::from_row(&row)?);
                    }

                    Ok(result)
                };
                QueryMethod::new(sig, fn_body, fetch_stmt, self.use_async)
            }
            QueryCommand::Exec
            | QueryCommand::ExecRows
            | QueryCommand::ExecResult
            | QueryCommand::ExecLastId => {
                let sig = quote! { fn #ident_name(&mut self, #arg) -> anyhow::Result<()> };
                let fetch_stmt = quote! {
                    #client.execute(#ident_const_name, &[#fields_list])
                };
                let fn_body = quote! {
                    Ok(())
                };
                QueryMethod::new(sig, fn_body, fetch_stmt, self.use_async)
            } // _ => quote! {},
        };

        query_method.to_token_stream()
    }
}

impl ToTokens for TypeQuery {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code().to_token_stream());
    }
}

struct QueryMethod {
    sig: TokenStream,
    fetch_stmt: TokenStream,
    fn_body: TokenStream,
    use_async: bool,
}

impl QueryMethod {
    fn new(
        sig: TokenStream,
        fn_body: TokenStream,
        fetch_stmt: TokenStream,
        use_async: bool,
    ) -> Self {
        Self {
            sig,
            fetch_stmt,
            fn_body,
            use_async,
        }
    }
}

impl ToTokens for QueryMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fn_code;
        let sig = self.sig.clone();
        let fn_body = self.fn_body.clone();
        let fetch_stmt = self.fetch_stmt.clone();
        if self.use_async {
            fn_code = quote! {
                pub(crate) async #sig {
                    #fetch_stmt.await?;
                    #fn_body
                }
            }
        } else {
            fn_code = quote! {
                pub(crate) #sig {
                    #fetch_stmt?;
                    #fn_body
                }
            }
        }

        tokens.extend(fn_code);
    }
}
