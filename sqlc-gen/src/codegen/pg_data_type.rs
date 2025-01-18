use crate::codegen::get_punct_from_char_tokens;
use crate::codegen::type_enum::enum_name;
use proc_macro2::TokenStream;
use quote::ToTokens;
use sqlc_sqlc_community_neoeinstein_prost::plugin;
use std::fmt;
use std::hash::Hash;

/// A wrapper around a `String` representing a generic data type.
/// TODO: remove if not necessary
#[derive(Debug, Clone)]
pub(crate) struct DataType(pub String);

impl ToTokens for DataType {
    /// Converts the `DataType` into a `TokenStream`, which is used for procedural macros.
    ///
    /// Breaks the `DataType`'s string into characters and converts each character
    /// to the corresponding punctuation token via `get_punct_from_char_tokens`.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.0.chars().map(|c| get_punct_from_char_tokens(c)));
    }
}

/// A wrapper around a `String` representing a PostgreSQL data type.
#[derive(Debug, Clone, Hash, PartialEq)]
pub struct PgDataType(pub String);

impl ToTokens for PgDataType {
    /// Converts the `PgDataType` into a `TokenStream` for procedural macros.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.as_data_type().into_token_stream());
    }
}

/// Enum representing PostgreSQL types, used for mapping to corresponding Rust types.
#[derive(Debug, Clone, Hash, PartialEq)]
pub enum PgType {
    SmallInt,    // `i16`
    Integer,     // `i32`
    BigInt,      // `i64`
    Real,        // `f32`
    Float,       // `f64`
    Boolean,     // `bool`
    Json,        // `serde_json::Value`
    Bytea,       // `Vec<u8>`
    Date,        // `time::Date`
    Time,        // `time::Time`
    Timestamp,   // `time::PrimitiveDateTime`
    Timestamptz, // `time::OffsetDateTime`
    Interval,    // `i64`
    Text,        // `String`
    Uuid,        // `uuid::Uuid`
    Inet,        // `cidr::IpInet`
    Cidr,        // `cidr::IpCidr`
    MacAddr,     // `eui48::MacAddress`
    Hstore,      // `std::collections::HashMap<String, Option<String>>`
    BitVec,      // `bit_vec::BitVec`
    Point,       // `geo_types::Point<f64>`
    Box,         // `geo_types::Rect<f64>`
    Path,        // `geo_types::LineString<f64>`
    Unknown,     // Fallback for unknown types, corresponds to `String`
}

impl PgType {
    /// Maps a PostgreSQL type name (`s`) to a `PgType` variant.
    ///
    /// This function takes a string representing a PostgreSQL type and returns the corresponding
    /// `PgType` variant. If no match is found, it returns `PgType::Unknown`.
    pub fn from_str(s: &str) -> Self {
        match s {
            "smallint" | "int2" | "pg_catalog.int2" | "smallserial" | "serial2"
            | "pg_catalog.serial2" => PgType::SmallInt,
            "integer" | "int" | "int4" | "pg_catalog.int4" | "serial" | "serial4"
            | "pg_catalog.serial4" => PgType::Integer,
            "bigint" | "int8" | "pg_catalog.int8" | "bigserial" | "serial8"
            | "pg_catalog.serial8" => PgType::BigInt,
            "real" | "float4" | "pg_catalog.float4" => PgType::Real,
            "float" | "double precision" | "float8" | "pg_catalog.float8" => PgType::Float,
            "boolean" | "bool" | "pg_catalog.bool" => PgType::Boolean,
            "json" | "jsonb" => PgType::Json,
            "bytea" | "blob" | "pg_catalog.bytea" => PgType::Bytea,
            "date" => PgType::Date,
            "pg_catalog.time" | "pg_catalog.timez" => PgType::Time,
            "pg_catalog.timestamp" => PgType::Timestamp,
            "pg_catalog.timestamptz" | "timestamptz" => PgType::Timestamptz,
            "interval" | "pg_catalog.interval" => PgType::Interval,
            "text" | "pg_catalog.varchar" | "pg_catalog.bpchar" | "string" | "citext" | "ltree"
            | "lquery" | "ltxtquery" => PgType::Text,
            "uuid" => PgType::Uuid,
            "inet" => PgType::Inet,
            "cidr" => PgType::Cidr,
            "macaddr" | "macaddr8" => PgType::MacAddr,
            "hstore" => PgType::Hstore,
            "bit" | "varbit" | "pg_catalog.bit" | "pg_catalog.varbit" => PgType::BitVec,
            "point" => PgType::Point,
            "box" => PgType::Box,
            "path" => PgType::Path,
            _ => PgType::Unknown,
        }
    }

    /// Maps the `PgType` to its corresponding Rust type string.
    ///
    /// This method returns a `String` that represents the equivalent Rust type for the PostgreSQL
    /// type (e.g., `i16`, `f64`, `String`, etc.). For unknown types, it returns `String`.
    pub fn to_rust_type(&self) -> String {
        match *self {
            PgType::SmallInt => "i16".to_string(),
            PgType::Integer => "i32".to_string(),
            PgType::BigInt => "i64".to_string(),
            PgType::Real => "f32".to_string(),
            PgType::Float => "f64".to_string(),
            PgType::Boolean => "bool".to_string(),
            PgType::Json => "serde_json::Value".to_string(),
            PgType::Bytea => "Vec<u8>".to_string(),
            PgType::Date => "time::Date".to_string(),
            PgType::Time => "time::Time".to_string(),
            PgType::Timestamp => "time::PrimitiveDateTime".to_string(),
            PgType::Timestamptz => "time::OffsetDateTime".to_string(),
            PgType::Interval => "i64".to_string(),
            PgType::Text => "String".to_string(),
            PgType::Uuid => "uuid::Uuid".to_string(),
            PgType::Inet => "cidr::IpInet".to_string(),
            PgType::Cidr => "cidr::IpCidr".to_string(),
            PgType::MacAddr => "eui48::MacAddress".to_string(),
            PgType::Hstore => "std::collections::HashMap<String, Option<String>>".to_string(),
            PgType::BitVec => "bit_vec::BitVec".to_string(),
            PgType::Point => "geo_types::Point<f64>".to_string(),
            PgType::Box => "geo_types::Rect<f64>".to_string(),
            PgType::Path => "geo_types::LineString<f64>".to_string(),
            PgType::Unknown => "String".to_string(), // Fallback for unknown types
        }
    }

    /// Generates the name for an enum type based on the schema and default schema.
    ///
    /// Checks if a given type is an enum and returns the corresponding name by
    /// looking up the enum in the provided schemas. If no enum is found, it falls back to the
    /// Rust type string.
    pub fn generate_enum_name(
        &self,
        schemas: &[plugin::Schema],
        s: &str,
        default_schema: &str,
    ) -> String {
        if let PgType::Unknown = *self {
            // Handling enums from schemas (non-primitive types)
            if let Some((matching_enum, schema)) = schemas
                .iter()
                .filter_map(|schema| {
                    if schema.name == "pg_catalog" || schema.name == "information_schema" {
                        None
                    } else {
                        schema
                            .enums
                            .iter()
                            .find(|e| e.name == s)
                            .map(|e| (e, schema))
                    }
                })
                .next()
            {
                return enum_name(&matching_enum.name, &schema.name, default_schema);
            }
        }
        self.to_rust_type() // Fallback if no enum is found
    }
}

impl PgDataType {
    /// Converts the `PgDataType` to a `DataType`.
    ///
    /// Creates a `DataType` by wrapping the string of the `PgDataType`.
    pub fn as_data_type(&self) -> DataType {
        DataType(self.to_string())
    }

    /// Constructs a `PgDataType` from a column in a schema.
    ///
    /// Extracts the type from a `plugin::Column` and uses the schema information
    /// to map the type to its corresponding Rust type string.
    pub fn from_col(
        col: &plugin::Column,
        schemas: &[plugin::Schema],
        default_schema: &str,
    ) -> Self {
        Self::from_str(
            col.r#type.as_ref().unwrap().name.as_str(),
            &schemas,
            &default_schema,
        )
    }

    /// Maps a string representation of a PostgreSQL type to a `PgDataType`.
    ///
    /// Uses the `PgType::from_str` to convert the PostgreSQL type string into a
    /// `PgType`. If the type is unknown, it generates the appropriate enum name if applicable.
    pub fn from_str(s: &str, schemas: &[plugin::Schema], default_schema: &str) -> Self {
        // Get the corresponding PgType from string
        let pg_type = PgType::from_str(s);

        // Check if it's an enum type and get the appropriate enum name if applicable
        let rust_type = match pg_type {
            PgType::Unknown => pg_type.generate_enum_name(schemas, s, default_schema),
            _ => pg_type.to_rust_type(),
        };

        PgDataType(rust_type)
    }
}

impl fmt::Display for PgDataType {
    /// Implements the `fmt::Display` trait for `PgDataType`.
    ///
    /// Returns the string representation of the `PgDataType`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlc_sqlc_community_neoeinstein_prost::plugin;

    // A simple helper function to create test schemas
    fn create_test_schema() -> Vec<plugin::Schema> {
        vec![
            plugin::Schema {
                name: "public".to_string(),
                enums: vec![plugin::Enum {
                    name: "my_enum".to_string(),
                    vals: vec!["value1".to_string(), "value2".to_string()],
                    comment: "".to_string(),
                }],
                ..Default::default()
            },
            plugin::Schema {
                name: "pg_catalog".to_string(),
                enums: vec![], // pg_catalog schema has no enums
                ..Default::default()
            },
        ]
    }

    // Test conversion of PostgreSQL types to Rust types
    #[test]
    fn test_pg_type_to_rust_type() {
        assert_eq!(PgType::SmallInt.to_rust_type(), "i16");
        assert_eq!(PgType::Integer.to_rust_type(), "i32");
        assert_eq!(PgType::BigInt.to_rust_type(), "i64");
        assert_eq!(PgType::Real.to_rust_type(), "f32");
        assert_eq!(PgType::Float.to_rust_type(), "f64");
        assert_eq!(PgType::Boolean.to_rust_type(), "bool");
        assert_eq!(PgType::Json.to_rust_type(), "serde_json::Value");
        assert_eq!(PgType::Bytea.to_rust_type(), "Vec<u8>");
        assert_eq!(PgType::Date.to_rust_type(), "time::Date");
        assert_eq!(PgType::Time.to_rust_type(), "time::Time");
        assert_eq!(PgType::Timestamp.to_rust_type(), "time::PrimitiveDateTime");
        assert_eq!(PgType::Timestamptz.to_rust_type(), "time::OffsetDateTime");
        assert_eq!(PgType::Interval.to_rust_type(), "i64");
        assert_eq!(PgType::Text.to_rust_type(), "String");
        assert_eq!(PgType::Uuid.to_rust_type(), "uuid::Uuid");
        assert_eq!(PgType::Inet.to_rust_type(), "cidr::IpInet");
        assert_eq!(PgType::Cidr.to_rust_type(), "cidr::IpCidr");
        assert_eq!(PgType::MacAddr.to_rust_type(), "eui48::MacAddress");
        assert_eq!(
            PgType::Hstore.to_rust_type(),
            "std::collections::HashMap<String, Option<String>>"
        );
        assert_eq!(PgType::BitVec.to_rust_type(), "bit_vec::BitVec");
        assert_eq!(PgType::Point.to_rust_type(), "geo_types::Point<f64>");
        assert_eq!(PgType::Box.to_rust_type(), "geo_types::Rect<f64>");
        assert_eq!(PgType::Path.to_rust_type(), "geo_types::LineString<f64>");
        assert_eq!(PgType::Unknown.to_rust_type(), "String");
    }

    // Test conversion from string to PgType
    #[test]
    fn test_pg_type_from_str() {
        assert_eq!(PgType::from_str("smallint"), PgType::SmallInt);
        assert_eq!(PgType::from_str("integer"), PgType::Integer);
        assert_eq!(PgType::from_str("bigint"), PgType::BigInt);
        assert_eq!(PgType::from_str("real"), PgType::Real);
        assert_eq!(PgType::from_str("boolean"), PgType::Boolean);
        assert_eq!(PgType::from_str("json"), PgType::Json);
        assert_eq!(PgType::from_str("text"), PgType::Text);
        assert_eq!(PgType::from_str("uuid"), PgType::Uuid);
        assert_eq!(PgType::from_str("unknown_type"), PgType::Unknown);
    }

    // Test generate_enum_name for an unknown PgType
    #[test]
    fn test_generate_enum_name_for_unknown_type() {
        let schemas = create_test_schema();
        let pg_type = PgType::Unknown;
        let result = pg_type.generate_enum_name(&schemas, "my_enum", "public");

        // For an unknown type, it should return the name of the enum from the schema.
        assert_eq!(result, "my_enum");
    }

    // Test generate_enum_name fallback behavior
    #[test]
    fn test_generate_enum_name_fallback() {
        let schemas = create_test_schema();
        let pg_type = PgType::Unknown;
        let result = pg_type.generate_enum_name(&schemas, "non_existing_enum", "public");

        // No matching enum, so it should fall back to the Rust type string.
        assert_eq!(result, "String");
    }

    // Test PgDataType conversion from column
    #[test]
    fn test_pg_data_type_from_col() {
        let schemas = create_test_schema();
        let col = plugin::Column {
            r#type: Some(plugin::Identifier {
                name: "text".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let pg_data_type = PgDataType::from_col(&col, &schemas, "public");

        // It should map the "text" type to a Rust String type.
        assert_eq!(pg_data_type.0, "String");
    }

    // Test PgDataType from_str conversion
    #[test]
    fn test_pg_data_type_from_str() {
        let schemas = create_test_schema();
        let result = PgDataType::from_str("boolean", &schemas, "public");

        // The "boolean" type should map to "bool".
        assert_eq!(result.0, "bool");
    }

    // Test Display implementation for PgDataType
    #[test]
    fn test_pg_data_type_display() {
        let pg_data_type = PgDataType("String".to_string());

        // It should display the inner string.
        assert_eq!(format!("{}", pg_data_type), "String");
    }
}
