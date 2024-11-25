use super::{get_ident, DataType, PgDataType};
use crate::codegen::TypeStruct;
use check_keyword::CheckKeyword;
use convert_case::{Case, Casing};
use core::panic;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::str::FromStr;
use strum_macros::EnumString;

fn escape(s: &str) -> String {
    if s.is_keyword() {
        format!("s_{s}")
    } else {
        s.to_string()
    }
}

fn param_name(p: &crate::plugin::Parameter) -> String {
    let column = p.column.as_ref().expect("column not found");

    if !column.name.is_empty() {
        column.name.to_case(convert_case::Case::Snake)
    } else {
        format!("dollar_{}", p.number)
    }
}

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
    #[strum(serialize = ":batchexec")]
    BatchExec,
    #[strum(serialize = ":batchmany")]
    BatchMany,
    #[strum(serialize = ":batchone")]
    BatchOne,
}

impl QueryCommand {
    pub fn has_return_value(&self) -> bool {
        match *self {
            Self::One | Self::Many | Self::BatchOne | Self::BatchMany => true,
            _ => false,
        }
    }

    pub fn is_batch(&self) -> bool {
        match *self {
            Self::BatchExec | Self::BatchMany | Self::BatchOne => true,
            _ => false,
        }
    }

    pub fn is_one(&self) -> bool {
        *self == Self::One || *self == Self::BatchOne
    }
}

#[derive(Default, Debug, Clone)]
pub struct QueryValue {
    name: String,
    typ: Option<PgDataType>,
    pub type_struct: Option<TypeStruct>,
    is_batch: bool,
}

impl QueryValue {
    pub fn new<S: Into<String>>(
        name: S,
        typ: Option<PgDataType>,
        type_struct: Option<TypeStruct>,
        is_batch: bool,
    ) -> Self {
        Self {
            name: name.into(),
            typ,
            type_struct,
            is_batch,
        }
    }

    pub fn from_query_params(
        params: &[crate::plugin::Parameter],
        schemas: &[crate::plugin::Schema],
        default_schema: &str,
        query_name: &str,
        qpl: usize,
        is_batch: bool,
    ) -> Option<Self> {
        if params.len() == 1 && qpl != 0 {
            let p = params.first().unwrap();
            let col = p.column.as_ref().unwrap();
            Some(Self::new(
                escape(&param_name(p)),
                Some(PgDataType::from_col(col, &schemas, &default_schema)),
                None,
                is_batch,
            ))
        } else if params.len() > 1 {
            let type_struct = TypeStruct::from_params(query_name, params, schemas, default_schema);
            Some(Self::new("arg", None, Some(type_struct.clone()), is_batch))
        } else {
            None
        }
    }

    pub fn from_query_columns(
        columns: &[crate::plugin::Column],
        schemas: &[crate::plugin::Schema],
        default_schema: &str,
        structs: &[TypeStruct],
        query_cmd: &QueryCommand,
        query_name: &str,
        is_batch: bool,
    ) -> (Option<Self>, bool) {
        if columns.len() == 1 {
            let col = columns.first().unwrap();
            (
                Some(Self::new(
                    "",
                    Some(PgDataType::from_col(col, &schemas, &default_schema)),
                    None,
                    is_batch,
                )),
                false,
            )
        } else if query_cmd.has_return_value() {
            let found_struct = structs
                .iter()
                .find(|s| s.has_same_fields(&columns, schemas, default_schema));

            let mut new_struct = false;
            let gs = match found_struct {
                None => {
                    new_struct = true;
                    TypeStruct::from_columns(query_name, columns, schemas, default_schema)
                }
                Some(gs) => gs.clone(),
            };

            (
                Some(QueryValue::new("", None, Some(gs), is_batch)),
                new_struct,
            )
        } else {
            (None, false)
        }
    }

    pub fn get_type(&self) -> DataType {
        if let Some(typ) = &self.typ {
            typ.as_data_type()
        } else if let Some(ref type_struct) = self.type_struct {
            type_struct.data_type()
        } else {
            panic!("QueryValue neither has `typ` specified nor `type_struct`");
        }
    }

    pub fn get_type_tokens(&self) -> TokenStream {
        let data_type = &self.get_type();
        quote!(#data_type)
    }

    pub fn generate_fields_list(&self) -> TokenStream {
        let mut fields_list = quote! {};
        if self.typ.is_some() {
            let ident_name = get_ident(&self.name);
            fields_list = quote! { &#ident_name };
        } else if let Some(ref type_struct) = self.type_struct {
            let ident_name = get_ident(&self.name);
            let fields = type_struct
                .fields
                .iter()
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

    pub(crate) fn generate_code(&self, is_batch: bool) -> TokenStream {
        if !self.name.is_empty() {
            let ident_type = self.get_type_tokens();
            let ident_name = get_ident(&self.name);
            let arg_toks = if is_batch {
                let ident_name = get_ident(format!("{}_list", self.name).as_str());
                quote! {
                    #ident_name: &'b [#ident_type]
                }
            } else {
                quote! {
                    #ident_name: #ident_type
                }
            };
            arg_toks
        } else if self.typ.is_some() || self.type_struct.is_some() {
            let ident_type = &self.get_type_tokens();
            quote! {
                #ident_type
            }
        } else {
            quote! {}
        }
    }
}

impl ToTokens for QueryValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code(self.is_batch));
    }
}

#[derive(Default)]
pub struct TypeQuery {
    name: String,
    cmd: String,
    arg: Option<QueryValue>,
    ret: Option<QueryValue>,
    use_async: bool,
}

impl TypeQuery {
    pub fn new<S: Into<String>>(
        name: S,
        cmd: S,
        arg: Option<QueryValue>,
        ret: Option<QueryValue>,
        use_async: bool,
    ) -> Self {
        Self {
            name: name.into(),
            cmd: cmd.into(),
            arg,
            ret,
            use_async,
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
        let arg = self.arg.clone().unwrap_or_default();
        let fields_list = arg.generate_fields_list();
        let command = self.command();
        let client = quote!(client);

        let sig_fn = quote!(fn #ident_name(client: &impl sqlc_core::DBTX, #arg));

        let query_method = match command {
            QueryCommand::One => {
                let ret = self.ret.as_ref().unwrap();
                let sig = quote! { #sig_fn -> Result<#ret, sqlc_core::Error> };
                let fetch_stmt = quote! {
                    let row = #client.query_one(#ident_const_name, &[#fields_list])
                };
                let fn_body = quote! {
                    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
                };
                QueryMethod::new(sig, fn_body, fetch_stmt, self.use_async)
            }
            QueryCommand::Many => {
                let ret = self.ret.as_ref().unwrap();
                let sig = quote! { #sig_fn -> Result<Vec<#ret>, sqlc_core::Error> };
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
                let sig = quote! { #sig_fn -> Result<(), sqlc_core::Error> };
                let fetch_stmt = quote! {
                    #client.execute(#ident_const_name, &[#fields_list])
                };
                let fn_body = quote! {
                    Ok(())
                };
                QueryMethod::new(sig, fn_body, fetch_stmt, self.use_async)
            }
            QueryCommand::BatchMany | QueryCommand::BatchOne | QueryCommand::BatchExec => {
                let fut_ret = if command.has_return_value() {
                    let ret = self.ret.as_ref().unwrap();
                    if command.is_one() {
                        quote!(#ret)
                    } else {
                        quote! {
                            std::pin::Pin<Box<futures::stream::Iter<
                                impl std::iter::Iterator<
                                    Item = Result<
                                        Result<#ret, tokio_postgres::Error>,
                                        sqlc_core::Error,
                                    >,
                                >,
                            >>>
                        }
                    }
                } else {
                    quote!(())
                };
                let arg_name_str = arg.name.clone();
                let arg_name = get_ident(&arg_name_str);
                let arg_list = get_ident(format!("{arg_name_str}_list").as_str());
                let arg_type = arg.get_type();
                let sig = quote! {
                    fn #ident_name<'a, 'b, T: sqlc_core::DBTX>(client: &'a T, #arg) -> Result<
                        impl futures::Stream<
                                Item = std::pin::Pin<
                                    Box<impl futures::Future<Output = Result<#fut_ret, sqlc_core::Error>> + use<'a, 'b, T>>,
                                >,
                            > + use<'a, 'b, T>,
                        sqlc_core::Error,
                    >
                };
                let stmt = quote! {
                    let stmt = #client.prepare(#ident_const_name)
                };
                let fn_res = match command {
                    QueryCommand::BatchExec => {
                        quote! {
                            client.execute(&stmt, &[#fields_list]).await?;
                            Ok(())
                        }
                    }
                    QueryCommand::BatchOne => {
                        quote! {
                            let row = client
                                .query_one(
                                    &stmt,
                                    &[#fields_list],
                                )
                                .await?;
                            Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
                        }
                    }
                    QueryCommand::BatchMany => {
                        quote! {
                            let rows = client.query(&stmt, &[#fields_list]).await?;
                            let result = rows.into_iter().map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)));

                            Ok(Box::pin(futures::stream::iter(result)))
                        }
                    }
                    _ => unimplemented!(),
                };
                let fn_body = quote! {
                    let fut = move |#arg_name: &'b #arg_type| {
                        let stmt = stmt.clone();
                        Box::pin(async move {
                            #fn_res
                        })
                    };
                    Ok(futures::stream::iter(#arg_list.iter().map(fut)))
                };
                QueryMethod::new(sig, fn_body, stmt, self.use_async)
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
        let sig = &self.sig;
        let fn_body = &self.fn_body;
        let fetch_stmt = &self.fetch_stmt;
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
