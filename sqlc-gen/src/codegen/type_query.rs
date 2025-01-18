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

/// Helper function to create a `TypeStruct` from parameters.
fn create_type_struct(
    query_name: &str,
    params: &[crate::plugin::Parameter],
    schemas: &[crate::plugin::Schema],
    default_schema: &str,
    options: &crate::codegen::Options,
) -> TypeStruct {
    StructType::Params(StructParams {
        name: query_name,
        params,
        schemas,
        default_schema,
        options,
    })
    .into()
}

/// Helper function to find or create a `TypeStruct` based on column data.
fn find_or_create_struct(
    structs: &mut Vec<TypeStruct>,
    query_name: &str,
    columns: &[crate::plugin::Column],
    schemas: &[crate::plugin::Schema],
    default_schema: &str,
    options: &crate::codegen::Options,
) -> TypeStruct {
    structs
        .iter()
        .find(|s| s.has_same_fields(columns, schemas, default_schema))
        .cloned()
        .unwrap_or_else(|| {
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
        })
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
            let p = &params[0];
            let col = p.column.as_ref()?;
            return Some(Self::new(
                escape(&param_name(p)),
                Some(PgDataType::from_col(col, schemas, default_schema)),
                None,
                is_batch,
            ));
        }

        if params.len() > 1 {
            let type_struct =
                create_type_struct(query_name, params, schemas, default_schema, options);
            structs.push(type_struct.clone());

            return Some(Self::new("arg", None, Some(type_struct), is_batch));
        }

        None
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
            let col = &columns[0];
            return Some(Self::new(
                "",
                Some(PgDataType::from_col(col, schemas, default_schema)),
                None,
                is_batch,
            ));
        }

        if query_cmd.has_return_value() {
            let gs = find_or_create_struct(
                structs,
                query_name,
                columns,
                schemas,
                default_schema,
                options,
            );

            return Some(QueryValue::new("", None, Some(gs), is_batch));
        }

        None
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

    fn query_arg(&self) -> TokenStream {
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
    pub name: String,
    pub cmd: String,
    pub arg: Option<QueryValue>,
    pub ret: Option<QueryValue>,
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
        arg.query_arg()
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
                I::Item: std::borrow::Borrow<#arg_type> + Send + 'a,
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

#[cfg(test)]
mod tests {
    use super::*; // Import functions being tested.
    use crate::codegen::Options; // Assuming `Options` comes from codegen.
    use crate::codegen::QueryCommand;
    use crate::codegen::QueryValue;
    use crate::plugin::{Column, Parameter}; // Assuming these are in the plugin module.

    // Test for `escape` function
    #[test]
    fn test_escape() {
        // Test case for SQL keywords
        assert_eq!(escape("select"), "s_select");
        assert_eq!(escape("from"), "s_from");

        // Test case for non-SQL keywords
        assert_eq!(escape("column_name"), "column_name");
        assert_eq!(escape("non_keyword"), "non_keyword");
    }

    // Test for `param_name` function
    #[test]
    fn test_param_name() {
        let param_with_name = Parameter {
            column: Some(Column {
                name: "my_column".to_string(),
                ..Default::default()
            }),
            number: 1,
            ..Default::default()
        };

        let param_without_name = Parameter {
            column: Some(Column {
                name: "".to_string(),
                ..Default::default()
            }),
            number: 1,
            ..Default::default()
        };

        // Test case when `column.name` is non-empty
        assert_eq!(param_name(&param_with_name), "my_column");

        // Test case when `column.name` is empty
        assert_eq!(param_name(&param_without_name), "dollar_1");
    }

    // Test for `create_type_struct`
    #[test]
    fn test_create_type_struct() {
        let params = vec![
            Parameter {
                column: Some(Column {
                    name: "col1".to_string(),
                    ..Default::default()
                }),
                number: 1,
                ..Default::default()
            },
            Parameter {
                column: Some(Column {
                    name: "col2".to_string(),
                    ..Default::default()
                }),
                number: 2,
                ..Default::default()
            },
        ];
        let schemas = vec![]; // Provide the necessary schemas if required for your case
        let default_schema = "public";
        let options = Options::default(); // Assuming there's a default method

        let result = create_type_struct("test_query", &params, &schemas, default_schema, &options);

        // Assert that the TypeStruct was created and matches expectations.
        // This depends on the specifics of `TypeStruct` and its `ToTokens` trait.
        // Replace with appropriate checks.
        assert!(matches!(result, TypeStruct::Params(_)));
    }

    // Test for `find_or_create_struct`
    #[test]
    fn test_find_or_create_struct() {
        let mut structs = Vec::new();
        let query_name = "test_query";
        let columns = vec![
            Column {
                name: "col1".to_string(),
                ..Default::default()
            },
            Column {
                name: "col2".to_string(),
                ..Default::default()
            },
        ];
        let schemas = vec![]; // Provide the necessary schemas if required for your case
        let default_schema = "public";
        let options = Options::default(); // Assuming there's a default method

        // Test case where the struct is not found and a new one is created
        let result1 = find_or_create_struct(
            &mut structs,
            query_name,
            &columns,
            &schemas,
            default_schema,
            &options,
        );
        assert_eq!(structs.len(), 1); // A new struct should have been created

        // Test case where the struct already exists
        let result2 = find_or_create_struct(
            &mut structs,
            query_name,
            &columns,
            &schemas,
            default_schema,
            &options,
        );
        assert_eq!(structs.len(), 1); // No new struct should be created

        // Ensure that both results are the same (the struct was found, not recreated)
        assert_eq!(result1, result2);
    }

    // Test for enum string parsing (EnumString from strum)
    #[test]
    fn test_query_command_from_str() {
        assert_eq!(QueryCommand::from_str(":one").unwrap(), QueryCommand::One);
        assert_eq!(QueryCommand::from_str(":many").unwrap(), QueryCommand::Many);
        assert_eq!(QueryCommand::from_str(":exec").unwrap(), QueryCommand::Exec);
        assert_eq!(
            QueryCommand::from_str(":execresult").unwrap(),
            QueryCommand::ExecResult
        );
        assert_eq!(
            QueryCommand::from_str(":execrows").unwrap(),
            QueryCommand::ExecRows
        );
        assert_eq!(
            QueryCommand::from_str(":execlastid").unwrap(),
            QueryCommand::ExecLastId
        );
        assert_eq!(
            QueryCommand::from_str(":batchexec").unwrap(),
            QueryCommand::BatchExec
        );
        assert_eq!(
            QueryCommand::from_str(":batchmany").unwrap(),
            QueryCommand::BatchMany
        );
        assert_eq!(
            QueryCommand::from_str(":batchone").unwrap(),
            QueryCommand::BatchOne
        );
    }

    // Test for invalid string
    #[test]
    fn test_query_command_from_str_invalid() {
        assert!(QueryCommand::from_str(":invalid").is_err());
    }

    // Test for `has_return_value` method
    #[test]
    fn test_has_return_value() {
        assert!(QueryCommand::One.has_return_value());
        assert!(QueryCommand::Many.has_return_value());
        assert!(QueryCommand::BatchOne.has_return_value());
        assert!(QueryCommand::BatchMany.has_return_value());
        assert!(!QueryCommand::Exec.has_return_value());
        assert!(!QueryCommand::ExecResult.has_return_value());
        assert!(!QueryCommand::ExecRows.has_return_value());
        assert!(!QueryCommand::ExecLastId.has_return_value());
        assert!(!QueryCommand::BatchExec.has_return_value());
    }

    // Test for `is_batch` method
    #[test]
    fn test_is_batch() {
        assert!(!QueryCommand::One.is_batch());
        assert!(!QueryCommand::Many.is_batch());
        assert!(!QueryCommand::Exec.is_batch());
        assert!(!QueryCommand::ExecResult.is_batch());
        assert!(!QueryCommand::ExecRows.is_batch());
        assert!(!QueryCommand::ExecLastId.is_batch());
        assert!(QueryCommand::BatchExec.is_batch());
        assert!(QueryCommand::BatchMany.is_batch());
        assert!(QueryCommand::BatchOne.is_batch());
    }

    // Test for `client_method_name` method
    #[test]
    fn test_client_method_name() {
        assert_eq!(
            QueryCommand::One.client_method_name().to_string(),
            "query_one"
        );
        assert_eq!(QueryCommand::Many.client_method_name().to_string(), "query");
        assert_eq!(
            QueryCommand::Exec.client_method_name().to_string(),
            "execute"
        );
        assert_eq!(
            QueryCommand::ExecResult.client_method_name().to_string(),
            "execute"
        );
        assert_eq!(
            QueryCommand::ExecRows.client_method_name().to_string(),
            "execute"
        );
        assert_eq!(
            QueryCommand::ExecLastId.client_method_name().to_string(),
            "execute"
        );
        assert_eq!(
            QueryCommand::BatchExec.client_method_name().to_string(),
            "batch_execute"
        );
        assert_eq!(
            QueryCommand::BatchMany.client_method_name().to_string(),
            "batch_many"
        );
        assert_eq!(
            QueryCommand::BatchOne.client_method_name().to_string(),
            "batch_one"
        );
    }

    #[test]
    fn test_query_value_new() {
        let query_value = QueryValue::new("test_name", None, None, true);

        assert_eq!(query_value.name, "test_name");
        assert!(query_value.typ.is_none());
        assert!(query_value.type_struct.is_none());
        assert!(query_value.is_batch);
    }

    #[test]
    fn test_query_value_from_query_params_single_param() {
        // Setup a mock parameter
        let params = vec![Parameter {
            number: 1,
            column: Some(Column {
                name: "column_name".to_string(),
                ..Default::default()
            }),
        }];
        let schemas = vec![]; // Empty schemas for now
        let default_schema = "default_schema";
        let mut structs = vec![];
        let query_name = "test_query";
        let qpl = 1;
        let is_batch = false;
        let options = Options::default();

        let query_value = QueryValue::from_query_params(
            &params,
            &schemas,
            default_schema,
            &mut structs,
            query_name,
            qpl,
            is_batch,
            &options,
        );

        assert!(query_value.is_some());
        let query_value = query_value.unwrap();
        assert_eq!(query_value.name, "column_name"); // Should match the column name
        assert_eq!(query_value.is_batch, is_batch);
    }

    #[test]
    fn test_query_value_from_query_params_multiple_params() {
        let params = vec![
            Parameter {
                number: 1,
                column: Some(Column {
                    name: "col1".to_string(),
                    ..Default::default()
                }),
            },
            Parameter {
                number: 2,
                column: Some(Column {
                    name: "col2".to_string(),
                    ..Default::default()
                }),
            },
        ];
        let schemas = vec![]; // Empty schemas for now
        let default_schema = "default_schema";
        let mut structs = vec![];
        let query_name = "test_query";
        let qpl = 0;
        let is_batch = false;
        let options = Options::default();

        let query_value = QueryValue::from_query_params(
            &params,
            &schemas,
            default_schema,
            &mut structs,
            query_name,
            qpl,
            is_batch,
            &options,
        );

        assert!(query_value.is_some());
        let query_value = query_value.unwrap();
        assert_eq!(query_value.name, "arg"); // As per `create_type_struct`
    }

    // #[test]
    // fn test_get_type() {
    //     let query_value = QueryValue::new("test_name", Some(PgDataType::Int4), None, false);
    //
    //     // Assuming `PgDataType::Int4` is properly implemented
    //     let data_type = query_value.get_type();
    //     assert_eq!(data_type, DataType::Int4);
    // }
    //
    // #[test]
    // fn test_generate_code_batch() {
    //     let query_value = QueryValue::new("test_name", Some(PgDataType::Int4), None, true);
    //
    //     let tokens = query_value.generate_code();
    //     assert!(tokens.to_string().contains("test_name_list"));
    // }
    //
    // #[test]
    // fn test_generate_code_non_batch() {
    //     let query_value = QueryValue::new("test_name", Some(PgDataType::Int4), None, false);
    //
    //     let tokens = query_value.generate_code();
    //     assert!(tokens.to_string().contains("test_name"));
    // }
    //
    // #[test]
    // fn test_generate_code_no_name() {
    //     let query_value = QueryValue::new("", Some(PgDataType::Int4), None, false);
    //
    //     let tokens = query_value.generate_code();
    //     assert!(tokens.to_string().is_empty()); // No name, so it should not generate any code
    // }

    #[test]
    fn test_query_value_from_query_columns_single_column() {
        let columns = vec![Column {
            name: "column_name".to_string(),
            ..Default::default()
        }];
        let schemas = vec![]; // Empty schemas for now
        let default_schema = "default_schema";
        let mut structs = vec![];
        let query_cmd = QueryCommand::One;
        let query_name = "test_query";
        let is_batch = false;
        let options = Options::default();

        let query_value = QueryValue::from_query_columns(
            &columns,
            &schemas,
            default_schema,
            &mut structs,
            &query_cmd,
            query_name,
            is_batch,
            &options,
        );

        assert!(query_value.is_some());
        let query_value = query_value.unwrap();
        assert_eq!(query_value.name, "");
        assert_eq!(query_value.is_batch, is_batch);
    }

    #[test]
    fn test_query_value_from_query_columns_multiple_columns() {
        let columns = vec![
            Column {
                name: "col1".to_string(),
                ..Default::default()
            },
            Column {
                name: "col2".to_string(),
                ..Default::default()
            },
        ];
        let schemas = vec![]; // Empty schemas for now
        let default_schema = "default_schema";
        let mut structs = vec![];
        let query_cmd = QueryCommand::One;
        let query_name = "test_query";
        let is_batch = false;
        let options = Options::default();

        let query_value = QueryValue::from_query_columns(
            &columns,
            &schemas,
            default_schema,
            &mut structs,
            &query_cmd,
            query_name,
            is_batch,
            &options,
        );

        assert!(query_value.is_some());
        let query_value = query_value.unwrap();
        assert_eq!(query_value.name, "");
        assert!(query_value.type_struct.is_some()); // Type struct should be created
    }

    // #[test]
    // fn test_type_query_new() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, false);
    //     let type_query = TypeQuery::new("test_query", ":one", Some(query_value), None, true);
    //
    //     assert_eq!(type_query.name, "test_query");
    //     assert_eq!(type_query.cmd, ":one");
    //     assert!(type_query.arg.is_some());
    //     assert!(type_query.ret.is_none());
    //     assert!(type_query.use_async);
    // }
    //
    // #[test]
    // fn test_constant_name() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, false);
    //     let type_query = TypeQuery::new("testQuery", ":one", Some(query_value), None, false);
    //
    //     // Test conversion to ScreamingSnake case
    //     assert_eq!(type_query.constant_name(), "TEST_QUERY");
    // }
    //
    // #[test]
    // fn test_name() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, false);
    //     let type_query = TypeQuery::new("testQuery", ":one", Some(query_value), None, false);
    //
    //     // Test conversion to Snake case
    //     assert_eq!(type_query.name(), "test_query");
    // }
    //
    // #[test]
    // fn test_command() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, false);
    //     let type_query = TypeQuery::new("testQuery", ":one", Some(query_value), None, false);
    //
    //     // Test parsing of the query command
    //     let command = type_query.command();
    //     assert_eq!(command, QueryCommand::One);
    // }
    //
    // #[test]
    // fn test_non_batch_fn_signature() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, false);
    //     let type_query = TypeQuery::new("testQuery", ":one", Some(query_value), None, false);
    //
    //     // Generate function signature for non-batch query
    //     let signature = type_query.non_batch_fn_signature();
    //     let expected_signature = quote! {
    //         fn test_query(
    //             client: mut impl sqlc_core::DBTX,
    //             arg_name: sqlc_core::DataType
    //         ) -> sqlc_core::Result<sqlc_core::DataType>
    //     };
    //     assert_eq!(signature.to_string(), expected_signature.to_string());
    // }
    //
    // #[test]
    // fn test_batch_fn_signature() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, true);
    //     let type_query = TypeQuery::new("testQuery", ":batchone", Some(query_value), None, false);
    //
    //     // Generate function signature for batch query
    //     let signature = type_query.batch_fn_signature();
    //     let expected_signature = quote! {
    //         fn test_query<'a, C, I>(client: &'a C, arg_name: I) -> sqlc_core::Result<
    //             sqlc_core::BatchStream<sqlc_core::DataType>
    //         >
    //         where
    //             C: sqlc_core::DBTX,
    //             I: IntoIterator + Send + 'a,
    //             I::Item: std::borrow::Borrow<sqlc_core::DataType> + Send + 'a,
    //     };
    //     assert_eq!(signature.to_string(), expected_signature.to_string());
    // }
    //
    // #[test]
    // fn test_fn_signature() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, false);
    //     let type_query = TypeQuery::new("testQuery", ":one", Some(query_value), None, false);
    //
    //     // Generate function signature based on command
    //     let signature = type_query.fn_signature();
    //     let expected_signature = quote! {
    //         fn test_query(
    //             client: mut impl sqlc_core::DBTX,
    //             arg_name: sqlc_core::DataType
    //         ) -> sqlc_core::Result<sqlc_core::DataType>
    //     };
    //     assert_eq!(signature.to_string(), expected_signature.to_string());
    // }
    //
    // #[test]
    // fn test_prepare_method() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, false);
    //     let type_query = TypeQuery::new("testQuery", ":one", Some(query_value), None, false);
    //
    //     // Prepare the method
    //     let query_method = type_query.prepare_method();
    //     let expected_fn_body = quote! {
    //         client.query_one(TEST_QUERY, arg_name)
    //     };
    //     assert_eq!(
    //         query_method.fn_body.to_string(),
    //         expected_fn_body.to_string()
    //     );
    // }
    //
    // #[test]
    // fn test_to_tokens() {
    //     let query_value = QueryValue::new("arg_name", Some(PgDataType::Int4), None, false);
    //     let type_query = TypeQuery::new("testQuery", ":one", Some(query_value), None, false);
    //
    //     let mut tokens = TokenStream::new();
    //     type_query.to_tokens(&mut tokens);
    //
    //     // Check if the generated tokens contain the expected method signature
    //     assert!(tokens.to_string().contains("fn test_query"));
    // }

    #[test]
    fn test_sync_function_signature() {
        let sig = quote! { fn example_query(client: impl sqlc_core::DBTX, param: i32) -> sqlc_core::Result<()> };
        let fn_body = quote! { client.query("SELECT * FROM example WHERE id = $1", param) };
        let query_method = QueryMethod::new(sig, fn_body, false);

        let mut tokens = TokenStream::new();
        query_method.to_tokens(&mut tokens);

        // Expected function signature and body for a sync function.
        let expected_tokens = quote! {
            pub(crate) fn example_query(client: impl sqlc_core::DBTX, param: i32) -> sqlc_core::Result<()> {
                client.query("SELECT * FROM example WHERE id = $1", param)
            }
        };

        // Assert that the generated tokens match the expected ones.
        assert_eq!(tokens.to_string(), expected_tokens.to_string());
    }

    #[test]
    fn test_async_function_signature() {
        let sig = quote! { fn example_query(client: impl sqlc_core::DBTX, param: i32) -> sqlc_core::Result<()> };
        let fn_body = quote! { client.query("SELECT * FROM example WHERE id = $1", param) };
        let query_method = QueryMethod::new(sig, fn_body, true);

        let mut tokens = TokenStream::new();
        query_method.to_tokens(&mut tokens);

        // Expected function signature and body for an async function.
        let expected_tokens = quote! {
            pub(crate) async fn example_query(client: impl sqlc_core::DBTX, param: i32) -> sqlc_core::Result<()> {
                client.query("SELECT * FROM example WHERE id = $1", param).await
            }
        };

        // Assert that the generated tokens match the expected ones.
        assert_eq!(tokens.to_string(), expected_tokens.to_string());
    }

    #[test]
    fn test_from_type_query_conversion() {
        // Assume you have a `TypeQuery` struct and `prepare_method` correctly sets up the signature and body
        let query = TypeQuery::new(
            "example_query",
            ":one",
            Some(QueryValue::new("param", None, None, false)),
            Some(QueryValue::new("result", None, None, false)),
            false,
        );

        let query_method: QueryMethod = (&query).into(); // Convert from TypeQuery to QueryMethod

        // Let's check the signature and body.
        let mut tokens = TokenStream::new();
        query_method.to_tokens(&mut tokens);

        // Expected tokens for the function generated by `TypeQuery` (based on provided data).
        let expected_tokens = quote! {
            pub(crate) fn example_query(client: impl sqlc_core::DBTX, param: i32) -> sqlc_core::Result<()> {
                client.query("SELECT * FROM example WHERE id = $1", param)
            }
        };

        // Assert that the generated function matches the expected function.
        assert_eq!(tokens.to_string(), expected_tokens.to_string());
    }
}
