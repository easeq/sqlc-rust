use options::Options;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use sqlc_sqlc_community_neoeinstein_prost::plugin;
use std::str::FromStr;
use syn::Ident;
use type_const::TypeConst;
use type_enum::TypeEnum;
use type_query::{QueryCommand, QueryValue, TypeQuery};
use type_struct::{StructTable, StructType, TypeStruct};

pub(crate) use multi_line::*;
pub(crate) use pg_data_type::*;

mod multi_line;
mod options;
mod pg_data_type;
mod type_const;
mod type_enum;
mod type_query;
mod type_struct;

/// Generates a `TokenStream` from a list of strings, where each string is
/// converted into a sequence of punctuation tokens.
///
/// # Arguments
/// - `list`: A slice of strings to be converted into `TokenStream`.
///
/// # Returns
/// An iterator that yields `TokenStream` objects, one for each string in the list.
pub(crate) fn list_tokenstream<'a>(list: &'a [String]) -> impl Iterator<Item = TokenStream> + 'a {
    list.iter().map(|item| {
        let mut tokens = TokenStream::new();
        for c in item.chars() {
            tokens.extend(crate::codegen::get_punct_from_char_tokens(c));
        }
        tokens
    })
}

/// Creates a new `Ident` from a string value, which is commonly used
/// in procedural macros for generating Rust code.
///
/// # Arguments
/// - `value`: A string to be converted into an `Ident` object.
///
/// # Returns
/// An `Ident` object representing the string.
pub fn get_ident(value: &str) -> Ident {
    format_ident!("{}", value)
}

/// Builds a `TypeQuery` from the provided query, schema, and options.
///
/// Parses a query, including its command and parameters, and generates
/// a corresponding `TypeQuery` for code generation.
///
/// # Arguments
/// - `query`: The query data structure that holds the query's name, command, and parameters.
/// - `schemas`: The schemas used to resolve references for the query.
/// - `default_schema`: The default schema to use when resolving the query.
/// - `structs`: A mutable vector that holds generated structs for tables.
/// - `options`: Configuration options that control various code generation settings.
///
/// # Returns
/// A `TypeQuery` representing the generated query.
fn build_query(
    query: &plugin::Query,
    schemas: &[plugin::Schema],
    default_schema: &str,
    structs: &mut Vec<TypeStruct>,
    options: &Options,
) -> TypeQuery {
    let query_cmd = QueryCommand::from_str(&query.cmd).expect("invalid query annotation");
    let is_batch = query_cmd.is_batch();

    // Query parameter limit, get it from the options
    let qpl = 3;
    let arg = QueryValue::from_query_params(
        &query.params,
        schemas,
        default_schema,
        structs,
        &query.name,
        qpl,
        is_batch,
        options,
    );

    let ret = QueryValue::from_query_columns(
        &query.columns,
        schemas,
        default_schema,
        structs,
        &query_cmd,
        &query.name,
        is_batch,
        options,
    );

    TypeQuery::new(&query.name, &query.cmd, arg, ret, options.use_async)
}

/// Builds enums from the provided schema and options. This function generates
/// `TypeEnum` instances for each enum defined in the schema.
///
/// # Arguments
/// - `schema`: The schema containing the enums to be built.
/// - `default_schema`: The default schema to use when resolving enums.
/// - `options`: Configuration options for code generation.
///
/// # Returns
/// An iterator that yields `TypeEnum` instances for each enum in the schema.
fn build_enums_from_schema<'a>(
    schema: &'a plugin::Schema,
    default_schema: &'a str,
    options: &'a Options,
) -> impl Iterator<Item = TypeEnum> + 'a {
    schema
        .enums
        .iter()
        .map(move |e| TypeEnum::from(e, &schema.name, default_schema, options))
}

/// Builds structs from the provided schema and options. This function generates
/// `TypeStruct` instances for each table in the schema.
///
/// # Arguments
/// - `schema`: The schema containing the tables to be built.
/// - `default_schema`: The default schema to use when resolving tables.
/// - `options`: Configuration options for code generation.
///
/// # Returns
/// An iterator that yields `TypeStruct` instances for each table in the schema.
fn build_structs_from_schema<'a>(
    schema: &'a plugin::Schema,
    default_schema: &'a str,
    options: &'a Options,
) -> impl Iterator<Item = TypeStruct> + 'a {
    schema.tables.iter().map(move |table| {
        StructType::Table(StructTable {
            table,
            schema,
            default_schema,
            options,
        })
        .into()
    })
}

/// A struct to hold partial code generation outputs, including enums, constants,
/// structs, and queries, generated from the schema and queries.
///
/// # Fields
/// - `enums`: A vector of `TypeEnum` instances representing the enums in the schema.
/// - `constants`: A vector of `TypeConst` instances representing the constants defined in queries.
/// - `structs`: A vector of `TypeStruct` instances representing the structs for tables.
/// - `queries`: A vector of `TypeQuery` instances representing the generated queries.
#[derive(Default)]
pub struct CodePartials<'a> {
    enums: Vec<TypeEnum>,
    constants: Vec<TypeConst<'a>>,
    structs: Vec<TypeStruct>,
    queries: Vec<TypeQuery>,
}

impl<'a> CodePartials<'a> {
    /// Sorts the generated code parts (`constants`, `enums`, `queries`, `structs`)
    /// alphabetically by their name. This ensures the output is consistently ordered.
    pub fn sort_all(&mut self) {
        self.constants
            .sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.enums.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.queries.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.structs.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
    }
}

impl<'a> From<&'a plugin::GenerateRequest> for CodePartials<'a> {
    /// Converts a `GenerateRequest` into a `CodePartials` struct containing the
    /// generated enums, structs, constants, and queries.
    ///
    /// Processes all schemas and queries from the request and
    /// builds the necessary code generation parts.
    ///
    /// # Arguments
    /// - `req`: The `GenerateRequest` containing schemas and queries to process.
    ///
    /// # Returns
    /// A `CodePartials` instance containing the generated code parts.
    fn from(req: &'a plugin::GenerateRequest) -> Self {
        let options: Options = req
            .settings
            .as_ref()
            .expect("could not find sqlc config")
            .into();
        let catalog = req.catalog.as_ref().unwrap();

        let mut code_partials = CodePartials::default();

        // Process schemas
        for schema in &catalog.schemas {
            if schema.name == "pg_catalog" || schema.name == "information_schema" {
                continue;
            }

            code_partials.enums.extend(build_enums_from_schema(
                schema,
                &catalog.default_schema,
                &options,
            ));

            code_partials.structs.extend(build_structs_from_schema(
                schema,
                &catalog.default_schema,
                &options,
            ));
        }

        // Process queries
        for query in &req.queries {
            if query.name.is_empty() || query.cmd.is_empty() {
                continue;
            }

            code_partials
                .constants
                .push(TypeConst::new(&query.name, &query.text));

            let q = build_query(
                &query,
                &catalog.schemas,
                &catalog.default_schema,
                &mut code_partials.structs,
                &options,
            );
            code_partials.queries.push(q);
        }

        code_partials.sort_all();

        code_partials
    }
}

impl<'a> ToTokens for CodePartials<'a> {
    /// Converts the `CodePartials` into a `TokenStream` for procedural macros,
    /// generating the code for enums, constants, structs, and queries.
    ///
    /// # Arguments
    /// - `tokens`: A mutable reference to the `TokenStream` that will hold
    ///   the generated code.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let generated_comment = MultiLine(
            r#"
            /// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
            /// DO NOT EDIT.
"#,
        );

        let Self {
            enums,
            structs,
            constants,
            queries,
        } = self;

        tokens.extend(quote! {
            #generated_comment
            #(#constants)*
            #(#enums)*
            #(#structs)*
            #(#queries)*
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;

    use quote::ToTokens;
    use sqlc_sqlc_community_neoeinstein_prost::plugin;

    // Test for `list_tokenstream`
    #[test]
    fn test_list_tokenstream() {
        let input = vec!["hello".to_string(), "world".to_string()];
        let token_streams: Vec<TokenStream> = list_tokenstream(&input).collect();

        assert_eq!(token_streams.len(), 2);
        // Check that the generated tokens are valid by inspecting the TokenStream (simplified)
        assert!(token_streams[0].to_string().contains("hello"));
        assert!(token_streams[1].to_string().contains("world"));
    }

    // Test for `get_ident`
    #[test]
    fn test_get_ident() {
        let ident = get_ident("some_ident");
        assert_eq!(ident.to_string(), "some_ident");
    }

    fn mock_query(
        name: &str,
        cmd: &str,
        params: Vec<plugin::Parameter>,
        columns: Vec<plugin::Column>,
    ) -> plugin::Query {
        plugin::Query {
            name: name.to_string(),
            cmd: cmd.to_string(),
            params,
            columns,
            ..Default::default()
        }
    }

    #[test]
    fn test_build_query() {
        let schema = plugin::Schema {
            comment: "Test schema".to_string(),
            name: "public".to_string(),
            tables: vec![plugin::Table {
                rel: Some(plugin::Identifier {
                    catalog: "my_catalog".to_string(),
                    schema: "public".to_string(),
                    name: "users".to_string(),
                }),
                columns: vec![
                    plugin::Column {
                        name: "id".to_string(),
                        not_null: true,
                        is_array: false,
                        comment: "User ID".to_string(),
                        length: 32,
                        is_named_param: false,
                        is_func_call: false,
                        scope: "public".to_string(),
                        table: None,
                        table_alias: "u".to_string(),
                        r#type: None,
                        is_sqlc_slice: false,
                        embed_table: None,
                        original_name: "id".to_string(),
                        unsigned: false,
                        array_dims: 0,
                    },
                    plugin::Column {
                        name: "name".to_string(),
                        not_null: false,
                        is_array: true,
                        comment: "User Name".to_string(),
                        length: 64,
                        is_named_param: false,
                        is_func_call: false,
                        scope: "public".to_string(),
                        table: None,
                        table_alias: "u".to_string(),
                        r#type: None,
                        is_sqlc_slice: false,
                        embed_table: None,
                        original_name: "name".to_string(),
                        unsigned: false,
                        array_dims: 1,
                    },
                ],
                comment: "Users table".to_string(),
            }],
            enums: vec![],
            composite_types: vec![],
        };

        let query = mock_query(
            "TestQuery",
            "SELECT * FROM users",
            vec![],
            vec![
                plugin::Column {
                    name: "id".to_string(),
                    not_null: true,
                    is_array: false,
                    comment: "User ID".to_string(),
                    length: 32,
                    is_named_param: false,
                    is_func_call: false,
                    scope: "public".to_string(),
                    table: None,
                    table_alias: "u".to_string(),
                    r#type: None,
                    is_sqlc_slice: false,
                    embed_table: None,
                    original_name: "id".to_string(),
                    unsigned: false,
                    array_dims: 0,
                },
                plugin::Column {
                    name: "name".to_string(),
                    not_null: false,
                    is_array: true,
                    comment: "User Name".to_string(),
                    length: 64,
                    is_named_param: false,
                    is_func_call: false,
                    scope: "public".to_string(),
                    table: None,
                    table_alias: "u".to_string(),
                    r#type: None,
                    is_sqlc_slice: false,
                    embed_table: None,
                    original_name: "name".to_string(),
                    unsigned: false,
                    array_dims: 1,
                },
            ],
        );

        let schemas = vec![schema]; // Mock schema with a table
        let default_schema = "public";
        let mut structs = vec![];
        let options = Options::default();

        let type_query = build_query(&query, &schemas, default_schema, &mut structs, &options);

        // Validate the generated query fields
        assert_eq!(type_query.name, "TestQuery");
        assert_eq!(type_query.cmd, "SELECT * FROM users");
    }

    #[test]
    fn test_build_enums_from_schema() {
        let schema = plugin::Schema {
            comment: "Test schema".to_string(),
            name: "public".to_string(),
            tables: vec![],
            enums: vec![plugin::Enum {
                name: "UserStatus".to_string(),
                vals: vec!["ACTIVE".to_string(), "INACTIVE".to_string()],
                comment: "User status enum".to_string(),
            }],
            composite_types: vec![],
        };

        let default_schema = "public";
        let options = Options::default();

        let enums: Vec<TypeEnum> =
            build_enums_from_schema(&schema, default_schema, &options).collect();

        assert_eq!(enums.len(), 1); // One enum in the schema
        assert_eq!(enums[0].name, "UserStatus");
        assert_eq!(enums[0].variants.len(), 2); // Enum has two values
        assert_eq!(
            enums[0].variants[0],
            type_enum::Variant::new("ACTIVE", "ACTIVE")
        );
        assert_eq!(
            enums[0].variants[1],
            type_enum::Variant::new("INACTIVE", "INACTIVE")
        );
    }

    #[test]
    fn test_build_structs_from_schema() {
        let schema = plugin::Schema {
            comment: "Test schema".to_string(),
            name: "public".to_string(),
            tables: vec![plugin::Table {
                rel: Some(plugin::Identifier {
                    catalog: "my_catalog".to_string(),
                    schema: "public".to_string(),
                    name: "users".to_string(),
                }),
                columns: vec![plugin::Column {
                    name: "id".to_string(),
                    not_null: true,
                    is_array: false,
                    comment: "User ID".to_string(),
                    length: 32,
                    is_named_param: false,
                    is_func_call: false,
                    scope: "public".to_string(),
                    table: None,
                    table_alias: "u".to_string(),
                    r#type: None,
                    is_sqlc_slice: false,
                    embed_table: None,
                    original_name: "id".to_string(),
                    unsigned: false,
                    array_dims: 0,
                }],
                comment: "Users table".to_string(),
            }],
            enums: vec![],
            composite_types: vec![],
        };

        let default_schema = "public";
        let options = Options::default();

        let structs: Vec<TypeStruct> =
            build_structs_from_schema(&schema, default_schema, &options).collect();

        assert_eq!(structs.len(), 1); // One table in the schema
        assert_eq!(structs[0].name(), "User");
        assert_eq!(structs[0].fields.len(), 1); // One column in the table
        assert_eq!(structs[0].fields[0].name, "id");
    }

    fn mock_generate_request() -> plugin::GenerateRequest {
        plugin::GenerateRequest {
            settings: Some(plugin::Settings::default()),
            catalog: Some(plugin::Catalog {
                schemas: vec![], // Mock schemas as needed
                default_schema: "public".to_string(),
                ..Default::default()
            }),
            queries: vec![
                plugin::Query {
                    name: "TestQuery1".to_string(),
                    cmd: "SELECT * FROM table1".to_string(),
                    params: vec![],
                    columns: vec![],
                    ..Default::default()
                },
                plugin::Query {
                    name: "TestQuery2".to_string(),
                    cmd: "SELECT * FROM table2".to_string(),
                    params: vec![],
                    columns: vec![],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_code_partials_creation() {
        let req = mock_generate_request();
        let code_partials = CodePartials::from(&req);

        // Check that the generated constants, queries, structs, and enums are as expected
        assert_eq!(code_partials.constants.len(), 2); // Two queries, hence two constants
        assert_eq!(code_partials.queries.len(), 2); // Two queries
    }

    #[test]
    fn test_code_partials_sorting() {
        let req = mock_generate_request();
        let mut code_partials = CodePartials::from(&req);

        // Check initial unsorted state
        assert_eq!(code_partials.queries[0].name, "TestQuery1");
        assert_eq!(code_partials.queries[1].name, "TestQuery2");

        // Sort the code parts
        code_partials.sort_all();

        // Check sorted state
        assert_eq!(code_partials.queries[0].name, "TestQuery1");
        assert_eq!(code_partials.queries[1].name, "TestQuery2");
    }

    #[test]
    fn test_code_partials_to_tokens() {
        let req = mock_generate_request();
        let code_partials = CodePartials::from(&req);

        let mut tokens = TokenStream::new();
        code_partials.to_tokens(&mut tokens);

        let generated_code = tokens.to_string();

        // Check that generated code contains the expected constants and queries
        assert!(generated_code.contains("TestQuery1"));
        assert!(generated_code.contains("SELECT * FROM table1"));
        assert!(generated_code.contains("TestQuery2"));
        assert!(generated_code.contains("SELECT * FROM table2"));
    }

    #[test]
    fn test_query_command_batch() {
        let query_cmd = QueryCommand::from_str("BATCH").expect("Invalid query command");
        assert!(query_cmd.is_batch());
    }

    #[test]
    fn test_query_command_non_batch() {
        let query_cmd = QueryCommand::from_str("SELECT").expect("Invalid query command");
        assert!(!query_cmd.is_batch());
    }
}
