use core::panic;
use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{format_ident, quote, ToTokens};
use serde::{Deserialize, Serialize};
use sqlc_sqlc_community_neoeinstein_prost::plugin;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use syn::Ident;
use type_const::TypeConst;
use type_enum::{enum_name, TypeEnum};
use type_query::{QueryCommand, QueryValue, TypeQuery};
use type_struct::TypeStruct;

mod type_const;
mod type_enum;
mod type_query;
mod type_struct;

pub fn get_ident(value: &str) -> Ident {
    format_ident!("{}", value)
}

pub fn get_punct_from_char(c: char) -> Punct {
    Punct::new(c, Spacing::Joint)
}

pub fn get_punct_from_char_tokens(c: char) -> TokenStream {
    get_punct_from_char(c).to_token_stream()
}

pub fn get_newline_tokens() -> TokenStream {
    let newline_char = char::from_u32(0x000A).unwrap();
    get_punct_from_char_tokens(newline_char)
}

#[derive(Debug, Clone)]
pub struct MultiLine<'a>(&'a str);

impl<'a> MultiLine<'a> {
    fn line_to_tokens(&self, line: &str) -> TokenStream {
        let mut tokens = TokenStream::new();
        for c in line.chars() {
            tokens.extend(get_punct_from_char_tokens(c));
        }

        tokens
    }

    fn lines_to_tokens(&self) -> (TokenStream, usize) {
        let mut tokens = TokenStream::new();
        let mut total_lines = 0;
        let lines = self.0.lines();
        for (i, line) in lines.enumerate() {
            if i > 0 {
                tokens.extend(get_newline_tokens());
            }

            tokens.extend(self.line_to_tokens(line));
            total_lines += 1;
        }

        (tokens, total_lines)
    }
}

impl<'a> ToTokens for MultiLine<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (lines_tokens, total_lines) = self.lines_to_tokens();

        if total_lines > 1 {
            tokens.extend(get_newline_tokens());
            tokens.extend(lines_tokens);
            tokens.extend(get_newline_tokens());
        } else {
            tokens.extend(lines_tokens);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultiLineString<'a>(&'a str);

impl<'a> ToTokens for MultiLineString<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        ['r', '#', '"'].map(|c| tokens.extend(get_punct_from_char_tokens(c)));
        tokens.extend(MultiLine(self.0).to_token_stream());
        ['"', '#', ' '].map(|c| tokens.extend(get_punct_from_char_tokens(c)));
    }
}

#[derive(Debug, Clone)]
pub struct DataType(String);

impl ToTokens for DataType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.0.chars().map(|c| get_punct_from_char_tokens(c)));
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct PgDataType(pub String);

impl ToTokens for PgDataType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.as_data_type().into_token_stream());
    }
}

impl PgDataType {
    pub fn as_data_type(&self) -> DataType {
        DataType(self.to_string())
    }

    pub fn from_col(
        col: &plugin::Column,
        schemas: &[plugin::Schema],
        default_schema: &str,
    ) -> Self {
        Self::from(
            col.r#type.as_ref().unwrap().name.as_str(),
            &schemas,
            &default_schema,
        )
    }

    pub fn from(s: &str, schemas: &[plugin::Schema], default_schema: &str) -> Self {
        let other_ret_type: String;
        let pg_data_type_string = match s {
            "smallint" | "int2" | "pg_catalog.int2" | "smallserial" | "serial2"
            | "pg_catalog.serial2" => "i16",

            "integer" | "int" | "int4" | "pg_catalog.int4" | "serial" | "serial4"
            | "pg_catalog.serial4" => "i32",

            "bigint" | "int8" | "pg_catalog.int8" | "bigserial" | "serial8"
            | "pg_catalog.serial8" => "i64",

            "real" | "float4" | "pg_catalog.float4" => "f32",
            "float" | "double precision" | "float8" | "pg_catalog.float8" => "f64",

            // "numeric" | "pg_catalog.numeric" | "money" => "",
            "boolean" | "bool" | "pg_catalog.bool" => "bool",

            "json" | "jsonb" => "serde_json::Value",

            "bytea" | "blob" | "pg_catalog.bytea" => "Vec<u8>",

            "date" => "time::Date",

            "pg_catalog.time" | "pg_catalog.timez" => "time::Time",

            "pg_catalog.timestamp" => "time::PrimitiveDateTime",
            "pg_catalog.timestamptz" | "timestamptz" => "time::OffsetDateTime",

            "interval" | "pg_catalog.interval" => "i64",
            "text" | "pg_catalog.varchar" | "pg_catalog.bpchar" | "string" | "citext" | "ltree"
            | "lquery" | "ltxtquery" => "String",

            "uuid" => "uuid::Uuid",
            "inet" => "cidr::IpInet",
            "cidr" => "cidr::IpCidr",
            "macaddr" | "macaddr8" => "eui48::MacAddress",

            "hstore" => "std::collections::HashMap<String, Option<String>>",
            "bit" | "varbit" | "pg_catalog.bit" | "pg_catalog.varbit" => "bit_vec::BitVec",
            "point" => "geo_types::Point<f64>",
            "box" => "geo_types::Rect<f64>",
            "path" => "geo_types::LineString<f64>",

            _ => {
                let res = schemas.iter().find_map(|schema| {
                    if schema.name == "pg_catalog" || schema.name == "information_schema" {
                        None
                    } else if let Some(matching_enum) = schema.enums.iter().find(|e| s == e.name) {
                        Some((matching_enum, schema))
                    } else {
                        None
                    }
                });

                other_ret_type = if let Some((matching_enum, schema)) = res {
                    enum_name(&matching_enum.name, &schema.name, default_schema)
                } else {
                    "String".to_string()
                };

                &other_ret_type
            }
        };

        PgDataType(pg_data_type_string.to_string())
    }
}

impl fmt::Display for PgDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Options {
    #[serde(default)]
    use_async: bool,

    #[serde(default)]
    use_deadpool: bool,
}

impl Options {
    pub fn use_deadpool(&self) -> bool {
        self.use_async && self.use_deadpool
    }
}

impl From<plugin::Settings> for Options {
    fn from(settings: plugin::Settings) -> Self {
        let codegen = settings
            .codegen
            .as_ref()
            .expect("codegen settings not defined in sqlc config");
        let options_str = match std::str::from_utf8(&codegen.options) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence in codegen options: {}", e),
        };

        let mut options = Options::default();
        if !options_str.is_empty() {
            options = serde_json::from_str(options_str).expect(
                format!(
                    "could not deserialize codegen options (valid object: {:?})",
                    serde_json::to_string(&Options::default())
                        .expect("could not convert options to json string"),
                )
                .as_str(),
            );
        }

        options
    }
}

#[derive(Default)]
pub struct CodePartials {
    enums: Vec<TypeEnum>,
    constants: Vec<TypeConst>,
    structs: Vec<TypeStruct>,
    queries: Vec<TypeQuery>,
}

impl CodePartials {
    fn sort_all(&mut self) {
        self.constants
            .sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.enums.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.queries.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.structs.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
    }
}

impl From<plugin::GenerateRequest> for CodePartials {
    fn from(req: plugin::GenerateRequest) -> Self {
        let options: Options = req.settings.expect("could not find sqlc config").into();
        let catalog = req.catalog.as_ref().unwrap();

        let mut code_partials = CodePartials::default();

        for schema in &catalog.schemas {
            if schema.name == "pg_catalog" || schema.name == "information_schema" {
                continue;
            }

            code_partials
                .enums
                .extend(build_enums_from_schema(schema, &catalog.default_schema));

            code_partials
                .structs
                .extend(build_structs_from_schema(schema, &catalog.default_schema));
        }

        for query in &req.queries {
            if query.name.is_empty() || query.cmd.is_empty() {
                continue;
            }

            code_partials.constants.push(query.into());

            let (query, associated_structs) = build_query(
                &query,
                &catalog.schemas,
                &catalog.default_schema,
                &code_partials.structs,
                options.use_async,
            );
            code_partials.queries.push(query);
            code_partials.structs.extend(associated_structs);
        }

        code_partials.sort_all();

        code_partials
    }
}

impl ToTokens for CodePartials {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let generated_comment = MultiLine(
            r#"
            /// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
            /// DO NOT EDIT.
"#,
        )
        .to_token_stream();

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

fn build_query(
    query: &plugin::Query,
    schemas: &[plugin::Schema],
    default_schema: &str,
    structs: &[TypeStruct],
    use_async: bool,
) -> (TypeQuery, Vec<TypeStruct>) {
    let mut associated_structs = vec![];

    // Query parameter limit, get it from the options
    let query_cmd = QueryCommand::from_str(&query.cmd).expect("invalid query annotation");
    let is_batch = query_cmd.is_batch();
    let qpl = 3;
    let arg = QueryValue::from_query_params(
        &query.params,
        schemas,
        default_schema,
        &query.name,
        qpl,
        is_batch,
    );

    if let Some(ref query_arg) = arg {
        if let Some(ref type_struct) = query_arg.type_struct {
            associated_structs.push(type_struct.clone());
        }
    }

    let (ret, has_new_struct) = QueryValue::from_query_columns(
        &query.columns,
        schemas,
        default_schema,
        structs,
        &query_cmd,
        &query.name,
        is_batch,
    );

    if has_new_struct {
        if let Some(ref query_ret) = ret {
            if let Some(ref type_struct) = query_ret.type_struct {
                associated_structs.push(type_struct.clone());
            }
        }
    }

    (
        TypeQuery::new(&query.name, &query.cmd, arg, ret, use_async),
        associated_structs,
    )
}

fn build_enums_from_schema(schema: &plugin::Schema, default_schema: &str) -> Vec<TypeEnum> {
    schema
        .enums
        .iter()
        .map(|e| TypeEnum::from(e, &schema.name, default_schema))
        .collect::<Vec<_>>()
}

fn build_structs_from_schema(schema: &plugin::Schema, default_schema: &str) -> Vec<TypeStruct> {
    schema
        .tables
        .iter()
        .map(|table| TypeStruct::from_table(table, schema, default_schema))
        .collect::<Vec<_>>()
}
