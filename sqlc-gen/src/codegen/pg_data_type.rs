use crate::codegen::get_punct_from_char_tokens;
use crate::codegen::type_enum::enum_name;
use proc_macro2::TokenStream;
use quote::ToTokens;
use sqlc_sqlc_community_neoeinstein_prost::plugin;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub(crate) struct DataType(pub String);

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
