use check_keyword::CheckKeyword;
use convert_case::Casing;
use core::panic;
use itertools::Itertools;
use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use syn::Ident;
use type_const::TypeConst;
use type_enum::TypeEnum;
use type_query::{QueryCommand, QueryValue, TypeQuery};
use type_struct::{StructField, StructType, TypeStruct};

mod type_const;
mod type_enum;
mod type_query;
mod type_struct;

pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

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

pub fn column_name(name: String, pos: i32) -> String {
    let col_name = match name.is_empty() {
        false => name.clone(),
        true => format!("_{}", pos),
    };

    col_name.to_case(convert_case::Case::Snake)
}

pub fn param_name(p: &plugin::Parameter) -> String {
    let Some(column) = p.column.clone() else {
        panic!("column not found");
    };

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
    col_table: Option<plugin::Identifier>,
    struct_table: Option<plugin::Identifier>,
    default_schema: String,
) -> bool {
    if let Some(table_id) = col_table {
        let mut schema = table_id.schema;
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
        let parts = self.0.split("::");
        for (i, part) in parts.enumerate() {
            if i > 0 {
                tokens.extend(get_punct_from_char_tokens(':'));
                tokens.extend(get_punct_from_char_tokens(':'));
            }
            tokens.extend(get_ident(part).into_token_stream());
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct PgDataType(pub String);

impl ToTokens for PgDataType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(DataType(self.to_string()).into_token_stream());
    }
}

impl fmt::Display for PgDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ident_str = match self.0.as_str() {
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
            "pg_catalog.timestampz" | "timestampz" => "time::PrimitiveDateTime",

            // "interval" | "pg_catalog.interval" => "",
            "text" | "pg_catalog.varchar" | "pg_catalog.bpchar" | "string" | "citext" | "ltree"
            | "lquery" | "ltxtquery" => "String",

            "uuid" => "uuid::Uuid",
            "inet" => "cidr::InetCidr",
            "cidr" => "cidr::InetAddr",
            "macaddr" | "macaddr8" => "eui48::MacAddress",

            _ => "String",
        };

        f.write_str(ident_str)
    }
}

#[derive(Default)]
pub struct CodeBuilder {
    req: plugin::CodeGenRequest,
}

impl CodeBuilder {
    pub fn new(req: plugin::CodeGenRequest) -> Self {
        Self { req }
    }
}

impl CodeBuilder {
    fn build_enums(&self) -> Vec<TypeEnum> {
        let catalog = self.req.catalog.clone().unwrap();
        let enums = catalog
            .schemas
            .clone()
            .into_iter()
            .filter_map(|schema| {
                if schema.name == "pg_catalog" || schema.name == "information_schema" {
                    None
                } else {
                    Some(
                        schema
                            .enums
                            .clone()
                            .into_iter()
                            .map(|e| {
                                let mut enum_name = e.name;
                                if schema.name != catalog.default_schema {
                                    enum_name = format!("{}_{enum_name}", schema.name);
                                }

                                TypeEnum::new(enum_name, e.vals)
                            })
                            .collect::<Vec<_>>(),
                    )
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        enums
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&a.name(), &b.name()))
            .collect::<Vec<_>>()
    }

    fn build_constants(&self) -> Vec<TypeConst> {
        let queries = self.req.queries.clone();
        queries
            .into_iter()
            .map(|q| TypeConst::new(q.name.clone(), q.text.clone()))
            .collect::<Vec<_>>()
    }

    fn build_structs(&self) -> Vec<TypeStruct> {
        let catalog = self.req.catalog.clone().unwrap();
        let mut structs = catalog
            .schemas
            .clone()
            .into_iter()
            .filter_map(|schema| {
                if schema.name == "pg_catalog" || schema.name == "information_schema" {
                    None
                } else {
                    let type_struct = schema
                        .tables
                        .clone()
                        .into_iter()
                        .map(|table| {
                            let mut table_name = table.rel.clone().unwrap().name;
                            if schema.name != catalog.default_schema {
                                table_name = format!("{}_{table_name}", schema.name);
                            }

                            let struct_name = pluralizer::pluralize(table_name.as_str(), 1, false);
                            let fields = table
                                .columns
                                .into_iter()
                                .map(|col| {
                                    StructField::new(
                                        col.name,
                                        0,
                                        PgDataType(col.r#type.unwrap().name),
                                        col.is_array,
                                        col.not_null,
                                    )
                                })
                                .collect::<Vec<_>>();

                            TypeStruct::new(
                                struct_name,
                                Some(plugin::Identifier {
                                    catalog: "".to_string(),
                                    schema: schema.name.clone(),
                                    name: table.rel.unwrap().name,
                                }),
                                StructType::Default,
                                fields,
                            )
                        })
                        .collect::<Vec<_>>();

                    Some(type_struct)
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        structs.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));

        structs
    }

    fn build_queries(&self, structs: Vec<TypeStruct>) -> (Vec<TypeQuery>, Vec<TypeStruct>) {
        let catalog = self.req.catalog.clone().unwrap();
        let mut associated_structs = vec![];
        let mut queries = self
            .req
            .queries
            .clone()
            .into_iter()
            .filter_map(|query| {
                if query.name.is_empty() || query.cmd.is_empty() {
                    None
                } else {
                    // Query parameter limit, get it from the options
                    let qpl = 3;
                    let arg: QueryValue;
                    let params = query.params.clone();
                    if params.len() == 1 && qpl != 0 {
                        let p = params.first().unwrap();
                        let col = p.column.clone().unwrap();
                        arg = QueryValue::new(
                            escape(&param_name(p)),
                            Some(col.r#type.unwrap().name.to_string()),
                            None,
                        );
                    } else {
                        let fields = params
                            .into_iter()
                            .map(|field| {
                                let col = field.column.unwrap();
                                StructField {
                                    name: col.name,
                                    number: field.number,
                                    is_array: col.is_array,
                                    not_null: col.not_null,
                                    data_type: PgDataType(col.r#type.unwrap().name),
                                }
                            })
                            .collect::<Vec<_>>();

                        let type_struct =
                            TypeStruct::new(query.name.clone(), None, StructType::Params, fields);
                        arg = QueryValue::new("arg", None, Some(type_struct.clone()));
                        associated_structs.push(type_struct);
                    }

                    let columns = query.columns.clone();
                    let mut ret: Option<QueryValue> = None;
                    if columns.len() == 1 {
                        let c = columns.first().unwrap();
                        ret = Some(QueryValue::new(
                            "",
                            Some(c.r#type.clone().unwrap().name.to_string()),
                            None,
                        ));
                    } else if QueryCommand::from_str(&query.cmd)
                        .expect("invalid query command")
                        .has_return_value()
                    {
                        let found_struct = structs.clone().into_iter().find(|s| {
                            if s.fields.len() != columns.len() {
                                false
                            } else {
                                s.fields.clone().into_iter().enumerate().all(|(i, field)| {
                                    let c = columns.get(i).unwrap();
                                    let same_name =
                                        field.name() == column_name(c.name.to_string(), i as i32);

                                    let same_type = field.data_type.to_string()
                                        == PgDataType(c.r#type.clone().unwrap().name).to_string();

                                    let same_table = same_table(
                                        c.table.clone(),
                                        s.table.clone(),
                                        catalog.default_schema.clone(),
                                    );

                                    same_name && same_type && same_table
                                })
                            }
                        });

                        let gs = match found_struct {
                            None => {
                                let fields = columns
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, col)| StructField {
                                        name: col.name,
                                        number: i as i32,
                                        is_array: col.is_array,
                                        not_null: col.not_null,
                                        data_type: PgDataType(col.r#type.unwrap().name),
                                    })
                                    .collect::<Vec<_>>();

                                let type_struct = TypeStruct::new(
                                    query.name.clone(),
                                    None,
                                    StructType::Row,
                                    fields,
                                );
                                associated_structs.push(type_struct.clone());
                                type_struct
                            }
                            Some(gs) => gs,
                        };

                        ret = Some(QueryValue::new("", None, Some(gs)));
                    }

                    Some(TypeQuery::new(
                        query.name.clone(),
                        query.cmd.clone(),
                        arg,
                        ret,
                    ))
                }
            })
            .collect::<Vec<_>>();

        queries.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        associated_structs.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));

        (queries, associated_structs)
    }

    pub fn generate_code(&self) -> TokenStream {
        let enums = self.build_enums();
        let constants = self.build_constants();
        let mut structs = self.build_structs();

        let (queries, associated_structs) = self.build_queries(structs.clone());

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

        let queries_impl = quote! {
            pub struct Queries {
                client: postgres::Client,
            }
            impl Queries {
                pub fn new(client: postgres::Client) -> Self {
                    Self { client }
                }

                #(#queries)*
            }
        };

        quote! {
            #generated_comment
            use postgres::{Error, Row};
            #(#constants)*
            #(#enums)*
            #(#structs)*
            #queries_impl
        }
    }
}
