use itertools::Itertools;
use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::fmt;
use syn::Ident;
use type_const::TypeConst;
use type_enum::TypeEnum;
use type_method::TypeMethod;
use type_struct::{StructField, StructType, TypeStruct};

mod type_const;
mod type_enum;
mod type_method;
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
        let double_quote_char = char::from_u32(0x0022).unwrap();
        let hashtag_char = char::from_u32(0x0023).unwrap();

        tokens.extend(get_punct_from_char_tokens('r'));
        tokens.extend(get_punct_from_char_tokens(hashtag_char));
        tokens.extend(get_punct_from_char_tokens(double_quote_char));

        tokens.extend(MultiLine(self.0).to_token_stream());

        tokens.extend(get_punct_from_char_tokens(double_quote_char));
        tokens.extend(get_punct_from_char_tokens(hashtag_char));
        tokens.extend(get_punct_from_char_tokens(' '));
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

#[derive(Debug, Clone)]
pub struct PgDataType(pub String);

impl ToTokens for PgDataType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(DataType(self.to_string()).into_token_stream());
    }
}

impl fmt::Display for PgDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ident_str = match self.0.as_str() {
            "smallint" | "int2" | "pg_catalog.int2" => "i16",
            "serial" | "smallserial" | "serial2" | "pg_catalog.serial2" => "u16",

            "integer" | "int" | "int4" | "pg_catalog.int4" => "i32",
            "serial4" | "pg_catalog.serial4" => "u32",

            "bigint" | "int8" | "pg_catalog.int8" => "i64",
            "bigserial" | "serial8" | "pg_catalog.serial8" => "u64",

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

pub trait CodePartial {
    fn of_type(self: &Self) -> String;
    fn generate_code(self: &Self) -> TokenStream;
}

struct PartialsBuilder {
    query: plugin::Query,
}

impl PartialsBuilder {
    pub fn new(query: plugin::Query) -> PartialsBuilder {
        Self { query }
    }

    fn create_struct_field_from_column(&self, col: plugin::Column, number: i32) -> StructField {
        StructField {
            name: col.name,
            number,
            is_array: col.is_array,
            not_null: col.not_null,
            data_type: PgDataType(col.r#type.unwrap().name),
        }
    }

    fn params_struct(&self) -> TypeStruct {
        let fields = self.query.params.clone();
        let fields = fields
            .into_iter()
            .map(|field| self.create_struct_field_from_column(field.column.unwrap(), field.number))
            .collect::<Vec<_>>();

        TypeStruct::new(self.query.name.clone(), StructType::Params, fields)
    }

    fn result_struct(&self) -> TypeStruct {
        let columns = self.query.columns.clone();
        let fields = columns
            .into_iter()
            .map(|col| self.create_struct_field_from_column(col, 0))
            .collect::<Vec<_>>();

        TypeStruct::new(self.query.name.clone(), StructType::Row, fields)
    }

    pub fn build(&self) -> Vec<Box<dyn CodePartial>> {
        let query_const = TypeConst::new(self.query.name.clone(), self.query.text.clone());
        let params_struct = self.params_struct();
        let result_struct = self.result_struct();
        let query_method = TypeMethod::new(
            self.query.name.clone(),
            self.query.cmd.clone(),
            query_const.clone(),
            params_struct.clone(),
            result_struct.clone(),
        );

        vec![
            Box::new(query_const),
            Box::new(params_struct),
            Box::new(result_struct),
            Box::new(query_method),
        ]
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
    fn build_enums(&self) -> Vec<TokenStream> {
        let catalog = self.req.catalog.clone().unwrap();
        let schemas = catalog.schemas;
        let enums = schemas
            .into_iter()
            .map(|schema| {
                schema
                    .enums
                    .into_iter()
                    .map(|e| TypeEnum::new(e.name, e.vals))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        enums
            .into_iter()
            .map(|e| e.generate_code())
            .collect::<Vec<_>>()
    }

    fn build_queries_struct(&self, queries: Vec<TokenStream>) -> TokenStream {
        let data_type = DataType("postgres::Client".to_string());

        quote! {
            pub struct Queries {
                client: #data_type
            }

            impl Queries {
                pub fn new(host: &str, port: &str, username: &str, password: &str, database_name: &str) -> Self {
                    let client = postgres::Client::connect(
                        format!(
                            "postgresql://{username}:{password}@{host}:{port}/{database_name}",
                            host = settings.host,
                            port = settings.port,
                            username = settings.username,
                            password = settings.password,
                            database_name = database_name,
                        )
                        .as_str(),
                        postgres::NoTls,
                    )
                    .unwrap();

                    Self { client }
                }

                #(#queries)*
            }
        }
    }

    fn build_queries(&self) -> Vec<TokenStream> {
        let build_query = |query: plugin::Query| -> Vec<Box<dyn CodePartial>> {
            PartialsBuilder::new(query).build()
        };

        self.req
            .queries
            .clone()
            .into_iter()
            .map(build_query)
            .flatten()
            .into_group_map_by(|e| e.of_type() == "method")
            .into_iter()
            .map(|(is_query_method, values)| {
                let tokens = values
                    .into_iter()
                    .map(|v| v.generate_code())
                    .collect::<Vec<_>>();
                if is_query_method {
                    self.build_queries_struct(tokens)
                } else {
                    quote! {
                        #(#tokens)*
                    }
                }
            })
            // .flatten()
            .collect::<Vec<_>>()
    }

    pub fn generate_code(&self) -> TokenStream {
        let enums_tokens = self.build_enums();
        let queries = self.build_queries();
        let generated_comment = MultiLine(
            r#"
            /// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
            /// DO NOT EDIT.
"#,
        )
        .to_token_stream();

        quote! {
            #generated_comment
            #(#enums_tokens)*
            #(#queries)*
        }
    }
}
