use crate::codegen::{get_ident, DataType, PgDataType, TypeStruct};
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
            fields_list = type_struct.to_pg_query_slice(&ident_name);
        } else {
            // add panic if necessary
            // panic!("QueryValue neither has `typ` specified nor `type_struct`");
        }

        fields_list
    }

    fn to_named_fn_arg_ref(&self) -> TokenStream {
        let ident_type = self.get_type_tokens();
        let ident_name = get_ident(format!("{}_list", self.name).as_str());
        quote! {
            #ident_name: &'a [#ident_type]
        }
    }

    fn to_named_fn_arg(&self) -> TokenStream {
        let ident_type = self.get_type_tokens();
        let ident_name = get_ident(&self.name);
        quote! {
            #ident_name: #ident_type
        }
    }

    fn to_fn_return_type(&self) -> TokenStream {
        let ident_type = &self.get_type_tokens();
        quote! {
            #ident_type
        }
    }

    pub(crate) fn generate_code(&self) -> TokenStream {
        if !self.name.is_empty() {
            if self.is_batch {
                self.to_named_fn_arg_ref()
            } else {
                self.to_named_fn_arg()
            }
        } else if self.typ.is_some() || self.type_struct.is_some() {
            self.to_fn_return_type()
        } else {
            quote! {}
        }
    }
}

impl ToTokens for QueryValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code());
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

    fn to_fn_input_signature(&self) -> TokenStream {
        let ident_name = get_ident(&self.name());
        let arg = self.arg.clone().unwrap_or_default();
        let client_mut = if self.use_async {
            quote!()
        } else {
            quote!(mut)
        };

        quote!(fn #ident_name(client: &#client_mut impl sqlc_core::DBTX, #arg))
    }

    fn to_field_list(&self) -> TokenStream {
        let arg = self.arg.clone().unwrap_or_default();
        arg.generate_fields_list()
    }

    fn method_for_one(&self) -> QueryMethod {
        let client = quote!(client);
        let ident_const_name = get_ident(&self.constant_name());

        let fields_list = self.to_field_list();

        let ret = self.ret.as_ref().unwrap();

        let sig_fn_input = self.to_fn_input_signature();
        let sig = quote! { #sig_fn_input -> sqlc_core::Result<#ret> };
        let fetch_stmt = quote! {
            let row = #client.query_one(#ident_const_name, &[#fields_list])
        };
        let fn_body = quote! {
            Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
        };

        QueryMethod::new(sig, fn_body, fetch_stmt, self.use_async)
    }

    fn method_for_many(&self) -> QueryMethod {
        let client = quote!(client);
        let ident_const_name = get_ident(&self.constant_name());

        let fields_list = self.to_field_list();

        let ret = self.ret.as_ref().unwrap();

        let sig_fn_input = self.to_fn_input_signature();
        let sig = quote! {
            #sig_fn_input -> sqlc_core::Result<
                impl std::iter::Iterator<Item = sqlc_core::Result<#ret>>
            >
        };
        let fetch_stmt = quote! {
            let rows = #client.query(#ident_const_name, &[#fields_list])
        };
        let fn_body = quote! {
            let iter = rows
                .into_iter()
                .map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)?));

            Ok(iter)
        };

        QueryMethod::new(sig, fn_body, fetch_stmt, self.use_async)
    }

    fn method_for_exec(&self) -> QueryMethod {
        let client = quote!(client);
        let ident_const_name = get_ident(&self.constant_name());

        let fields_list = self.to_field_list();

        let sig_fn_input = self.to_fn_input_signature();
        let sig = quote! { #sig_fn_input -> sqlc_core::Result<()> };
        let fetch_stmt = quote! {
            #client.execute(#ident_const_name, &[#fields_list])
        };
        let fn_body = quote! {
            Ok(())
        };

        QueryMethod::new(sig, fn_body, fetch_stmt, self.use_async)
    }

    fn method_for_batch(&self) -> QueryMethod {
        let client = quote!(client);
        let command = self.command();
        let fut_ret = if command.has_return_value() {
            let ret = self.ret.as_ref().unwrap();
            if command.is_one() {
                quote!(#ret)
            } else {
                quote! {
                    impl futures::Stream<
                        Item = sqlc_core::Result<
                            sqlc_core::Result<#ret>,
                        >,
                    >
                }
            }
        } else {
            quote!(())
        };

        let ident_const_name = get_ident(&self.constant_name());
        let ident_name = get_ident(&self.name());

        let arg = self.arg.clone().unwrap_or_default();
        let arg_name_str = arg.name.clone();
        let arg_name = get_ident(&arg_name_str);
        let arg_list = get_ident(format!("{arg_name_str}_list").as_str());
        let arg_type = arg.get_type();

        let fields_list = self.to_field_list();
        let sig = quote! {
            fn #ident_name<'a, T: sqlc_core::DBTX>(client: &'a T, #arg) -> sqlc_core::Result<
                impl futures::Stream<
                        Item = impl futures::Future<
                            Output = sqlc_core::Result<#fut_ret>
                        > + 'a,
                    > + 'a,
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
            let fut = move |#arg_name: &'a #arg_type| {
                let stmt = stmt.clone();
                Box::pin(async move {
                    #fn_res
                })
            };
            Ok(futures::stream::iter(#arg_list.iter().map(fut)))
        };
        QueryMethod::new(sig, fn_body, stmt, self.use_async)
    }
}

impl ToTokens for TypeQuery {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let query_method: QueryMethod = self.into();
        tokens.extend(query_method.to_token_stream());
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

impl From<&TypeQuery> for QueryMethod {
    fn from(query: &TypeQuery) -> Self {
        let query_method = match query.command() {
            QueryCommand::One => query.method_for_one(),
            QueryCommand::Many => query.method_for_many(),
            QueryCommand::Exec
            | QueryCommand::ExecRows
            | QueryCommand::ExecResult
            | QueryCommand::ExecLastId => query.method_for_exec(),
            QueryCommand::BatchMany | QueryCommand::BatchOne | QueryCommand::BatchExec => {
                query.method_for_batch()
            } // _ => quote! {},
        };

        query_method
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
