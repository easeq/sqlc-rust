use check_keyword::CheckKeyword;
use convert_case::Casing;
use core::panic;
use itertools::Itertools;
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
use type_struct::{StructField, StructType, TypeStruct};

mod type_const;
mod type_enum;
mod type_query;
mod type_struct;

pub fn get_ident(value: &str) -> Ident {
    format_ident!("{}", value)
}

pub fn get_batch_results_name(value: &str) -> String {
    format!("{}BatchResults", value.to_case(convert_case::Case::Pascal))
}

pub fn get_batch_results_ident(value: &str) -> Ident {
    format_ident!("{}", get_batch_results_name(value))
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

pub fn column_name(name: &str, pos: i32) -> String {
    match name.is_empty() {
        false => name.to_case(convert_case::Case::Snake),
        true => format!("_{}", pos),
    }
}

pub fn param_name(p: &plugin::Parameter) -> String {
    let column = p.column.as_ref().expect("column not found");

    if !column.name.is_empty() {
        column.name.to_case(convert_case::Case::Snake)
    } else {
        format!("dollar_{}", p.number)
    }
}

pub fn escape(s: &str) -> String {
    if s.is_keyword() {
        format!("s_{s}")
    } else {
        s.to_string()
    }
}

pub fn same_table(
    col_table: Option<&plugin::Identifier>,
    struct_table: Option<&plugin::Identifier>,
    default_schema: &str,
) -> bool {
    if let Some(table_id) = col_table {
        let mut schema = table_id.schema.as_str();
        if schema.is_empty() {
            schema = default_schema;
        }

        if let Some(f) = struct_table {
            table_id.catalog == f.catalog && schema == f.schema && table_id.name == f.name
        } else {
            false
        }
    } else {
        false
    }
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

    pub fn from(s: &str, schemas: &[plugin::Schema], default_schema: &str) -> PgDataType {
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

#[derive(Default)]
pub struct CodeBuilder {
    req: plugin::GenerateRequest,
    options: Options,
}

impl CodeBuilder {
    pub fn new(req: plugin::GenerateRequest) -> Self {
        let settings = req.settings.clone().expect("could not find sqlc config");
        let codegen = settings
            .codegen
            .expect("codegen settings not defined in sqlc config");
        let options_str = match std::str::from_utf8(&codegen.options) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence in codegen options: {}", e),
        };

        let mut code_builder = Self {
            req,
            options: Options::default(),
        };

        if !options_str.is_empty() {
            let options: Options = serde_json::from_str(options_str).expect(
                format!(
                    "could not deserialize codegen options (valid object: {:?})",
                    serde_json::to_string(&Options::default())
                        .expect("could not convert options to json string"),
                )
                .as_str(),
            );
            code_builder.options = options;
        }

        code_builder
    }
}

impl CodeBuilder {
    fn process_schemas(
        &self,
        schemas: &[plugin::Schema],
        default_schema: &str,
    ) -> (Vec<TypeEnum>, Vec<TypeStruct>) {
        let mut enums = vec![];
        let mut structs = vec![];
        for schema in schemas {
            if schema.name == "pg_catalog" || schema.name == "information_schema" {
                continue;
            }

            enums.extend(self.build_enums_from_schema(schema, default_schema));
            structs.extend(self.build_structs_from_schema(schema, default_schema));
        }

        enums.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        structs.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));

        (enums, structs)
    }

    fn build_enums_from_schema(
        &self,
        schema: &plugin::Schema,
        default_schema: &str,
    ) -> Vec<TypeEnum> {
        let mut enums = vec![];
        for e in &schema.enums {
            let enum_name = enum_name(&e.name, &schema.name, default_schema);

            enums.push(TypeEnum::new(enum_name, e.vals.clone()))
        }

        enums
    }

    fn build_structs_from_schema(
        &self,
        schema: &plugin::Schema,
        default_schema: &str,
    ) -> Vec<TypeStruct> {
        let mut structs = vec![];

        for table in &schema.tables {
            let table_rel = table.rel.as_ref().unwrap();
            let mut table_name = table_rel.name.clone();
            if schema.name != default_schema {
                table_name = format!("{}_{table_name}", schema.name);
            }

            let struct_name = pluralizer::pluralize(table_name.as_str(), 1, false);
            let fields = table
                .columns
                .iter()
                .map(|col| StructField::from(col, 0, &[schema.clone()], &default_schema))
                .collect::<Vec<_>>();

            structs.push(TypeStruct::new(
                struct_name,
                Some(plugin::Identifier {
                    catalog: "".to_string(),
                    schema: schema.name.clone(),
                    name: table_rel.name.clone(),
                }),
                StructType::Default,
                fields,
            ));
        }

        structs
    }

    fn process_queries(
        &self,
        queries: &[plugin::Query],
        schemas: &[plugin::Schema],
        default_schema: &str,
        structs: &[TypeStruct],
    ) -> (Vec<TypeConst>, Vec<TypeQuery>, Vec<TypeStruct>) {
        let mut constants = vec![];
        let mut type_queries = vec![];
        let mut associated_structs = vec![];
        for query in queries {
            if query.name.is_empty() || query.cmd.is_empty() {
                continue;
            }

            constants.push(TypeConst::new(&query.name, &query.text));

            let (query, structs) = self.build_query(&query, schemas, default_schema, structs);
            type_queries.push(query);
            associated_structs.extend(structs);
        }

        type_queries.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        associated_structs.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));

        (constants, type_queries, associated_structs)
    }

    fn build_query(
        &self,
        query: &plugin::Query,
        schemas: &[plugin::Schema],
        default_schema: &str,
        structs: &[TypeStruct],
    ) -> (TypeQuery, Vec<TypeStruct>) {
        let mut associated_structs = vec![];

        // Query parameter limit, get it from the options
        let query_cmd = QueryCommand::from_str(&query.cmd).expect("invalid query annotation");
        let is_batch = query_cmd.is_batch();
        let qpl = 3;
        let mut arg: Option<QueryValue> = None;
        let params = &query.params;
        if params.len() == 1 && qpl != 0 {
            let p = params.first().unwrap();
            let col = p.column.as_ref().unwrap();
            arg = Some(QueryValue::new(
                escape(&param_name(p)),
                Some(PgDataType::from(
                    col.r#type.as_ref().unwrap().name.as_str(),
                    &schemas,
                    &default_schema,
                )),
                None,
                is_batch,
            ));
        } else if params.len() > 1 {
            let fields = params
                .iter()
                .map(|field| {
                    StructField::from(
                        &field.column.as_ref().unwrap(),
                        field.number,
                        &schemas,
                        &default_schema,
                    )
                })
                .collect::<Vec<_>>();

            let type_struct = TypeStruct::new(&query.name, None, StructType::Params, fields);
            arg = Some(QueryValue::new(
                "arg",
                None,
                Some(type_struct.clone()),
                is_batch,
            ));
            associated_structs.push(type_struct);
        }

        let columns = &query.columns;
        let mut ret: Option<QueryValue> = None;
        if columns.len() == 1 {
            let col = columns.first().unwrap();
            ret = Some(QueryValue::new(
                "",
                Some(PgDataType::from(
                    col.r#type.as_ref().unwrap().name.as_str(),
                    &schemas,
                    &default_schema,
                )),
                None,
                is_batch,
            ));
        } else if query_cmd.has_return_value() {
            let found_struct = structs.iter().find(|s| {
                if s.fields.len() != columns.len() {
                    false
                } else {
                    s.fields
                        .iter()
                        .zip(columns.iter())
                        .enumerate()
                        .all(|(i, (field, c))| {
                            let same_name = field.name() == column_name(&c.name, i as i32);

                            let same_type = field.data_type.to_string()
                                == PgDataType::from(
                                    c.r#type.as_ref().unwrap().name.as_str(),
                                    &schemas,
                                    &default_schema,
                                )
                                .to_string();

                            let same_table =
                                same_table(c.table.as_ref(), s.table.as_ref(), &default_schema);

                            same_name && same_type && same_table
                        })
                }
            });

            let gs = match found_struct {
                None => {
                    let fields = columns
                        .iter()
                        .enumerate()
                        .map(|(i, col)| StructField::from(col, i as i32, &schemas, &default_schema))
                        .collect::<Vec<_>>();

                    let type_struct = TypeStruct::new(&query.name, None, StructType::Row, fields);
                    associated_structs.push(type_struct.clone());
                    type_struct
                }
                Some(gs) => gs.clone(),
            };

            ret = Some(QueryValue::new("", None, Some(gs), is_batch));
        }

        (
            TypeQuery::new(
                &query.name,
                &query.cmd,
                arg,
                ret,
                self.options.use_async,
                self.options.use_deadpool(),
            ),
            associated_structs,
        )
    }

    pub fn generate_code(&self) -> TokenStream {
        let catalog = self.req.catalog.as_ref().unwrap();
        let schemas = &catalog.schemas;
        let default_schema = &catalog.default_schema;
        let queries = &self.req.queries;

        let (enums, mut structs) = self.process_schemas(schemas, default_schema);
        let (constants, queries, associated_structs) =
            self.process_queries(queries, schemas, default_schema, &structs);

        structs.extend(associated_structs);
        structs.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));

        // TODO: below
        // let (enums, structs) = self.filter_unused_structs(enums, structs, queries);
        // validate_structs_and_enums

        let generated_comment = MultiLine(
            r#"
            /// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
            /// DO NOT EDIT.
"#,
        )
        .to_token_stream();

        quote! {
            #generated_comment
            #(#constants)*
            #(#enums)*
            #(#structs)*
            #(#queries)*
        }
    }
}
