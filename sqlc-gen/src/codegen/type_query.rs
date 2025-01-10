use crate::codegen::{
    get_ident,
    type_struct::{StructParams, StructRow, StructType},
    DataType, PgDataType, TypeStruct,
};
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

    pub fn client_method_name(&self) -> TokenStream {
        match *self {
            QueryCommand::One => quote!(query_one),
            QueryCommand::Many => quote!(query),
            QueryCommand::Exec
            | QueryCommand::ExecRows
            | QueryCommand::ExecResult
            | QueryCommand::ExecLastId => quote!(execute),
            QueryCommand::BatchOne => quote!(batch_one),
            QueryCommand::BatchMany => quote!(batch_many),
            QueryCommand::BatchExec => quote!(batch_execute),
        }
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

    pub(crate) fn from_query_params(
        params: &[crate::plugin::Parameter],
        schemas: &[crate::plugin::Schema],
        default_schema: &str,
        structs: &mut Vec<TypeStruct>,
        query_name: &str,
        qpl: usize,
        is_batch: bool,
        options: &crate::codegen::Options,
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
            let type_struct: TypeStruct = StructType::Params(StructParams {
                name: query_name,
                params,
                schemas,
                default_schema,
                options,
            })
            .into();

            structs.push(type_struct.clone());

            Some(Self::new("arg", None, Some(type_struct), is_batch))
        } else {
            None
        }
    }

    pub(crate) fn from_query_columns(
        columns: &[crate::plugin::Column],
        schemas: &[crate::plugin::Schema],
        default_schema: &str,
        structs: &mut Vec<TypeStruct>,
        query_cmd: &QueryCommand,
        query_name: &str,
        is_batch: bool,
        options: &crate::codegen::Options,
    ) -> Option<Self> {
        if columns.len() == 1 {
            let col = columns.first().unwrap();
            Some(Self::new(
                "",
                Some(PgDataType::from_col(col, &schemas, &default_schema)),
                None,
                is_batch,
            ))
        } else if query_cmd.has_return_value() {
            let found_struct = structs
                .iter()
                .find(|s| s.has_same_fields(&columns, schemas, default_schema));

            let gs = match found_struct {
                None => {
                    let s: TypeStruct = StructType::Row(StructRow {
                        name: query_name,
                        columns,
                        schemas,
                        default_schema,
                        options,
                    })
                    .into();
                    structs.push(s.clone());
                    s
                }
                .into(),
                Some(gs) => gs.clone(),
            };

            Some(QueryValue::new("", None, Some(gs), is_batch))
        } else {
            None
        }
    }

    fn get_type(&self) -> DataType {
        if let Some(typ) = &self.typ {
            typ.as_data_type()
        } else if let Some(ref type_struct) = self.type_struct {
            type_struct.data_type()
        } else {
            panic!("QueryValue neither has `typ` specified nor `type_struct`");
        }
    }

    fn get_type_tokens(&self) -> TokenStream {
        let data_type = &self.get_type();
        quote!(#data_type)
    }

    fn generate_fields_list(&self) -> TokenStream {
        if self.is_batch {
            let ident_name = get_ident(format!("{}_list", self.name).as_str());
            quote!(#ident_name)
        } else if self.typ.is_some() || self.type_struct.is_some() {
            let ident_name = get_ident(&self.name);
            quote!(#ident_name)
        } else {
            quote!(())
        }
    }

    fn to_named_fn_arg_ref(&self) -> TokenStream {
        let ident_name = get_ident(format!("{}_list", self.name).as_str());
        quote! {
            #ident_name: I
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

    fn generate_code(&self) -> TokenStream {
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

    fn query_arg(&self) -> TokenStream {
        let arg = self.arg.clone().unwrap_or_default();
        arg.generate_fields_list()
    }

    fn non_batch_fn_signature(&self) -> TokenStream {
        let ident_name = get_ident(&self.name());
        let arg = self.arg.clone().unwrap_or_default();
        let client_mut = if self.use_async {
            quote!()
        } else {
            quote!(mut)
        };

        let ret = self.ret.clone().unwrap_or_default();
        let ret_sig = match self.command() {
            QueryCommand::One => quote!(#ret),
            QueryCommand::Many => quote!(impl std::iter::Iterator<Item = sqlc_core::Result<#ret>>),
            QueryCommand::Exec
            | QueryCommand::ExecRows
            | QueryCommand::ExecResult
            | QueryCommand::ExecLastId => quote!(u64),
            _ => unimplemented!(),
        };

        quote! {
            fn #ident_name(
                client: &#client_mut impl sqlc_core::DBTX,
                #arg
            ) -> sqlc_core::Result<#ret_sig>
        }
    }

    fn batch_fn_signature(&self) -> TokenStream {
        let ident_name = get_ident(&self.name());
        let arg = self.arg.clone().unwrap_or_default();
        let arg_type = arg.get_type();
        let ret = self.ret.clone().unwrap_or_default();
        let fut_ret = match self.command() {
            QueryCommand::BatchOne => quote!(#ret),
            QueryCommand::BatchMany => quote!(sqlc_core::BoxStream<sqlc_core::Result<#ret>>),
            QueryCommand::BatchExec => quote!(()),
            _ => unimplemented!(),
        };

        quote! {
            fn #ident_name<'a, C, I>(client: &'a C, #arg) -> sqlc_core::Result<
                sqlc_core::BatchStream<#fut_ret>
            >
            where
                C: sqlc_core::DBTX,
                I: IntoIterator + Send + 'a,
                I::Item: std::borrow::Borrow<#arg_type> + 'a,
        }
    }

    fn fn_signature(&self) -> TokenStream {
        if self.command().is_batch() {
            self.batch_fn_signature()
        } else {
            self.non_batch_fn_signature()
        }
    }

    fn prepare_method(&self) -> QueryMethod {
        let client_method_name = self.command().client_method_name();
        let ident_const_name = get_ident(&self.constant_name());
        let query_arg = self.query_arg();
        let fn_body = quote! {
            client.#client_method_name(#ident_const_name, #query_arg)
        };

        QueryMethod::new(self.fn_signature(), fn_body, self.use_async)
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
    fn_body: TokenStream,
    use_async: bool,
}

impl QueryMethod {
    fn new(sig: TokenStream, fn_body: TokenStream, use_async: bool) -> Self {
        Self {
            sig,
            fn_body,
            use_async,
        }
    }
}

impl From<&TypeQuery> for QueryMethod {
    fn from(query: &TypeQuery) -> Self {
        query.prepare_method()
    }
}

impl ToTokens for QueryMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fn_code;
        let sig = &self.sig;
        let fn_body = &self.fn_body;
        if self.use_async {
            fn_code = quote! {
                pub(crate) async #sig {
                    #fn_body.await
                }
            }
        } else {
            fn_code = quote! {
                pub(crate) #sig {
                    #fn_body
                }
            }
        }

        tokens.extend(fn_code);
    }
}
