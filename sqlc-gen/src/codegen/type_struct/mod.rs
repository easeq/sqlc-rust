use crate::codegen::options::RuleType;
use crate::codegen::{plugin, DataType};
use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub(crate) use field::*;
pub(crate) use params::*;
pub(crate) use row::*;
pub(crate) use table::*;

mod field;
mod params;
mod row;
mod table;

/// Converts a column name to snake case if it is not empty.
/// If the name is empty, it returns a default formatted string using the position (`pos`).
fn column_name(name: &str, pos: i32) -> String {
    if name.is_empty() {
        format!("_{}", pos)
    } else {
        name.to_case(convert_case::Case::Snake)
    }
}

/// Checks if two tables (one from a column and one from a struct) match
/// based on their catalog, schema, and name, considering a default schema if needed.
fn same_table(
    col_table: Option<&plugin::Identifier>,
    struct_table: Option<&plugin::Identifier>,
    default_schema: &str,
) -> bool {
    match col_table {
        Some(col_id) => {
            let col_schema = if col_id.schema.is_empty() {
                default_schema
            } else {
                &col_id.schema
            };

            struct_table
                .map(|struct_id| {
                    col_id.catalog == struct_id.catalog
                        && col_schema == struct_id.schema
                        && col_id.name == struct_id.name
                })
                .unwrap_or(false)
        }
        None => false,
    }
}

// Enum representing different struct types.
pub enum StructType<'a> {
    Params(StructParams<'a>),
    Row(StructRow<'a>),
    Table(StructTable<'a>),
}

trait Struct<'a> {
    fn options(&self) -> &'a crate::codegen::Options;
    fn name(&self) -> String;
    fn fields(&self) -> Vec<StructField>;
    fn derive(&self) -> Option<Vec<String>> {
        if let Some(rules) = self.options().rules.as_ref() {
            Some(rules.derive_for(&self.name(), RuleType::Structs))
        } else {
            None
        }
    }

    fn attrs(&self) -> Option<Vec<String>> {
        if let Some(rules) = self.options().rules.as_ref() {
            Some(rules.container_attrs_for(&self.name(), RuleType::Structs))
        } else {
            None
        }
    }
    fn table_identifier(&self) -> Option<plugin::Identifier> {
        None
    }
}

// Implementing the `as_trait` method to return a reference to the `Struct` trait.
impl<'a> StructType<'a> {
    fn as_trait(&self) -> &dyn Struct<'a> {
        match self {
            Self::Params(t) => t,
            Self::Row(t) => t,
            Self::Table(t) => t,
        }
    }
}

// Converting `StructType` into `TypeStruct`.
impl<'a> From<StructType<'a>> for TypeStruct {
    fn from(struct_type: StructType<'a>) -> Self {
        let st = struct_type.as_trait();
        Self {
            name: st.name(),
            fields: st.fields(),
            table: st.table_identifier(),
            derive: st.derive().unwrap_or_default(),
            attrs: st.attrs().unwrap_or_default(),
        }
    }
}

/// Represents a struct type with metadata, including its name, fields, and associated options.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct TypeStruct {
    /// The name of the struct
    name: String,

    /// The table identifier (optional)
    pub table: Option<plugin::Identifier>,

    /// A list of fields belonging to the struct
    pub fields: Vec<StructField>,

    /// A list of derived attributes for the struct
    derive: Vec<String>,

    /// A list of additional attributes for the struct
    attrs: Vec<String>,
}

impl TypeStruct {
    /// Checks whether the struct's fields match the given columns from the database schema.
    ///
    /// This method compares the number of fields in the struct with the number of columns in the
    /// provided `columns` slice and verifies that each field matches the corresponding column.
    ///
    /// # Arguments
    /// * `columns` - The columns from the database schema to compare against.
    /// * `schemas` - The list of schemas available for comparison.
    /// * `default_schema` - The default schema to use when a schema is not specified.
    ///
    /// # Returns
    /// `true` if all fields match the columns, otherwise `false`.
    pub fn has_same_fields(
        &self,
        columns: &[plugin::Column],
        schemas: &[plugin::Schema],
        default_schema: &str,
    ) -> bool {
        if self.fields.len() != columns.len() {
            return false; // Return early if the number of fields does not match the number of columns
        }

        // Check if all fields match the corresponding columns
        self.fields
            .iter()
            .zip(columns.iter())
            .enumerate()
            .all(|(i, (field, col))| {
                field.matches_column(col, self.table.as_ref(), schemas, default_schema, i as i32)
            })
    }

    /// Returns the name of the struct.
    ///
    /// # Returns
    /// A reference to the name of the struct.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Returns the data type representation of the struct as a `DataType`.
    ///
    /// This is useful for generating type-safe SQL queries.
    ///
    /// # Returns
    /// A `DataType` representing the struct's name.
    pub(crate) fn data_type(&self) -> DataType {
        DataType(self.name.clone())
    }

    /// Generates the Rust code for this struct, including its fields, attributes, and derivations.
    ///
    /// This method generates the complete Rust struct code, including any derive attributes and
    /// user-defined attributes, as well as the struct's fields.
    ///
    /// # Returns
    /// A `TokenStream` representing the generated Rust code.
    fn generate_code(&self) -> TokenStream {
        if self.fields.is_empty() {
            return quote! {}; // Return an empty TokenStream if there are no fields
        }

        // Collect the tokens for struct fields, derive attributes, and other attributes
        let ident_struct = self.data_type();
        let fields_tokens = self.fields.iter().collect::<Vec<_>>();
        let derive_tokens = crate::codegen::list_tokenstream(&self.derive);
        let attr_tokens = crate::codegen::list_tokenstream(&self.attrs);

        // Generate the struct code with the necessary attributes and fields
        quote! {
            #[derive(
                sqlc_core::PostgresRow,
                sqlc_core::PostgresParams,
                #(#derive_tokens),*
            )]
            #(#attr_tokens)*
            pub struct #ident_struct {
                #(#fields_tokens),*
            }
        }
    }
}

impl ToTokens for TypeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::options::Options;
    use crate::codegen::PgDataType;

    #[test]
    fn test_column_name_non_empty() {
        let name = "MyColumn";
        let pos = 1;
        let result = column_name(name, pos);
        assert_eq!(result, "my_column"); // Expecting snake_case version of the name
    }

    #[test]
    fn test_column_name_empty() {
        let name = "";
        let pos = 42;
        let result = column_name(name, pos);
        assert_eq!(result, "_42"); // Expecting the format "_<pos>" when name is empty
    }

    #[test]
    fn test_same_table_match() {
        let col_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "schema".to_string(),
            name: "table_name".to_string(),
        };

        let struct_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "schema".to_string(),
            name: "table_name".to_string(),
        };

        let result = same_table(Some(&col_table), Some(&struct_table), "default_schema");
        assert!(result); // Tables should match
    }

    #[test]
    fn test_same_table_no_match_catalog() {
        let col_table = plugin::Identifier {
            catalog: "catalog1".to_string(),
            schema: "schema".to_string(),
            name: "table_name".to_string(),
        };

        let struct_table = plugin::Identifier {
            catalog: "catalog2".to_string(),
            schema: "schema".to_string(),
            name: "table_name".to_string(),
        };

        let result = same_table(Some(&col_table), Some(&struct_table), "default_schema");
        assert!(!result); // Catalogs do not match
    }

    #[test]
    fn test_same_table_no_match_schema() {
        let col_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "schema1".to_string(),
            name: "table_name".to_string(),
        };

        let struct_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "schema2".to_string(),
            name: "table_name".to_string(),
        };

        let result = same_table(Some(&col_table), Some(&struct_table), "default_schema");
        assert!(!result); // Schemas do not match
    }

    #[test]
    fn test_same_table_no_match_name() {
        let col_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "schema".to_string(),
            name: "table_name1".to_string(),
        };

        let struct_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "schema".to_string(),
            name: "table_name2".to_string(),
        };

        let result = same_table(Some(&col_table), Some(&struct_table), "default_schema");
        assert!(!result); // Table names do not match
    }

    #[test]
    fn test_same_table_no_match_col_table_none() {
        let col_table = None;
        let struct_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "schema".to_string(),
            name: "table_name".to_string(),
        };

        let result = same_table(col_table, Some(&struct_table), "default_schema");
        assert!(!result); // Column table is None
    }

    #[test]
    fn test_same_table_no_match_struct_table_none() {
        let col_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "schema".to_string(),
            name: "table_name".to_string(),
        };

        let struct_table = None;

        let result = same_table(Some(&col_table), struct_table, "default_schema");
        assert!(!result); // Struct table is None
    }

    #[test]
    fn test_same_table_default_schema() {
        let col_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "".to_string(),
            name: "table_name".to_string(),
        };

        let struct_table = plugin::Identifier {
            catalog: "catalog".to_string(),
            schema: "default_schema".to_string(),
            name: "table_name".to_string(),
        };

        let result = same_table(Some(&col_table), Some(&struct_table), "default_schema");
        assert!(result); // Schema is empty, default schema is used
    }

    // Helper function to create a mock Column
    fn mock_column(name: &str) -> plugin::Column {
        plugin::Column {
            name: name.to_string(),
            not_null: false,
            is_array: false,
            comment: "".to_string(),
            length: 0,
            is_named_param: false,
            is_func_call: false,
            scope: "".to_string(),
            table: None,
            table_alias: "".to_string(),
            r#type: None,
            is_sqlc_slice: false,
            embed_table: None,
            original_name: "".to_string(),
            unsigned: false,
            array_dims: 0,
        }
    }

    // Helper function to create a mock StructField
    fn mock_struct_field(name: &str) -> StructField {
        StructField {
            name: name.to_string(),
            data_type: PgDataType("String".to_string()), // Empty data type for simplicity
            ..Default::default()                         // Fills other fields with default values
        }
    }

    #[test]
    fn test_has_same_fields_match() {
        // Define a TypeStruct
        let struct_type = TypeStruct {
            name: "TestStruct".to_string(),
            table: None,
            fields: vec![mock_struct_field("column1"), mock_struct_field("column2")],
            derive: vec![],
            attrs: vec![],
        };

        // Define the columns to compare with
        let columns = vec![mock_column("column1"), mock_column("column2")];

        // Simulate schemas and default schema
        let schemas: Vec<plugin::Schema> = vec![];
        let default_schema = "public";

        // Test the matching fields
        assert!(struct_type.has_same_fields(&columns, &schemas, default_schema));
    }

    #[test]
    fn test_has_same_fields_no_match() {
        // Define a TypeStruct
        let struct_type = TypeStruct {
            name: "TestStruct".to_string(),
            table: None,
            fields: vec![mock_struct_field("column1"), mock_struct_field("column2")],
            derive: vec![],
            attrs: vec![],
        };

        // Define the columns with a mismatch
        let columns = vec![
            mock_column("column1"),
            mock_column("column3"), // column3 doesn't match column2
        ];

        // Simulate schemas and default schema
        let schemas: Vec<plugin::Schema> = vec![];
        let default_schema = "public";

        // Test the mismatch in fields
        assert!(!struct_type.has_same_fields(&columns, &schemas, default_schema));
    }

    #[test]
    fn test_name() {
        let struct_type = TypeStruct {
            name: "TestStruct".to_string(),
            table: None,
            fields: vec![],
            derive: vec![],
            attrs: vec![],
        };

        // Test that the name is returned correctly
        assert_eq!(struct_type.name(), "TestStruct");
    }

    #[test]
    fn test_data_type() {
        let struct_type = TypeStruct {
            name: "TestStruct".to_string(),
            table: None,
            fields: vec![],
            derive: vec![],
            attrs: vec![],
        };

        // Test that the data type is returned correctly
        assert_eq!(struct_type.data_type(), DataType("TestStruct".to_string()));
    }

    #[test]
    fn test_generate_code_with_fields() {
        let struct_type = TypeStruct {
            name: "TestStruct".to_string(),
            table: None,
            fields: vec![mock_struct_field("column1"), mock_struct_field("column2")],
            derive: vec!["Clone".to_string()],
            attrs: vec!["#[serde(rename_all=\"screaming-snake-case\")]".to_string()],
        };

        // Generate the code
        let generated_code = struct_type.generate_code();

        // Check that the generated code contains struct declaration with correct fields and attributes
        let expected_code = quote! {
            #[derive(
                sqlc_core::PostgresRow,
                sqlc_core::PostgresParams,
                Clone
            )]
            #[serde(rename_all="screaming-snake-case")]
            pub struct TestStruct {
                pub column_1: Option< String>,
                pub column_2: Option< String>,
            }
        };
        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn test_generate_code_without_fields() {
        let struct_type = TypeStruct {
            name: "TestStruct".to_string(),
            table: None,
            fields: vec![],
            derive: vec![],
            attrs: vec![],
        };

        // Generate the code for an empty struct
        let generated_code = struct_type.generate_code();

        // Check that the generated code is empty (since there are no fields)
        assert_eq!(generated_code.to_string(), "".to_string());
    }

    #[test]
    fn test_struct_type_as_trait_params() {
        let struct_params = StructParams {
            name: "Test",
            params: &[], // Assume empty params for simplicity
            default_schema: "public",
            schemas: &[],                 // Assume empty schemas for simplicity
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Params(struct_params);
        let trait_ref = struct_type.as_trait();

        assert_eq!(trait_ref.name(), "TestParams");
    }

    #[test]
    fn test_struct_type_as_trait_row() {
        let struct_row = StructRow {
            name: "Test",
            columns: &[], // Assume empty columns for simplicity
            default_schema: "public",
            schemas: &[],                 // Assume empty schemas for simplicity
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Row(struct_row);
        let trait_ref = struct_type.as_trait();

        assert_eq!(trait_ref.name(), "TestRow");
    }

    #[test]
    fn test_struct_type_as_trait_table() {
        let struct_table = StructTable {
            table: &plugin::Table {
                rel: Some(plugin::Identifier {
                    catalog: "catalog".to_string(),
                    schema: "public".to_string(),
                    name: "test_table".to_string(),
                }),
                columns: vec![], // Assume empty columns for simplicity
                comment: "".to_string(),
            },
            default_schema: "public",
            schema: &plugin::Schema {
                name: "public".to_string(),
                comment: "".to_string(),
                tables: vec![],          // Assume empty tables for simplicity
                enums: vec![],           // Assume empty enums for simplicity
                composite_types: vec![], // Assume empty composite types for simplicity
            },
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Table(struct_table);
        let trait_ref = struct_type.as_trait();

        assert_eq!(trait_ref.name(), "TestTable");
    }

    #[test]
    fn test_struct_type_from_struct_params() {
        let struct_params = StructParams {
            name: "Test",
            params: &[], // Assume empty params for simplicity
            default_schema: "public",
            schemas: &[],                 // Assume empty schemas for simplicity
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Params(struct_params);
        let type_struct: TypeStruct = struct_type.into();

        assert_eq!(type_struct.name, "TestParams");
    }

    #[test]
    fn test_struct_type_from_struct_row() {
        let struct_row = StructRow {
            name: "Test",
            columns: &[], // Assume empty columns for simplicity
            default_schema: "public",
            schemas: &[],                 // Assume empty schemas for simplicity
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Row(struct_row);
        let type_struct: TypeStruct = struct_type.into();

        assert_eq!(type_struct.name, "TestRow");
    }

    #[test]
    fn test_struct_type_from_struct_table() {
        let struct_table = StructTable {
            table: &plugin::Table {
                rel: Some(plugin::Identifier {
                    catalog: "catalog".to_string(),
                    schema: "public".to_string(),
                    name: "test_table".to_string(),
                }),
                columns: vec![], // Assume empty columns for simplicity
                comment: "".to_string(),
            },
            default_schema: "public",
            schema: &plugin::Schema {
                name: "public".to_string(),
                comment: "".to_string(),
                tables: vec![],          // Assume empty tables for simplicity
                enums: vec![],           // Assume empty enums for simplicity
                composite_types: vec![], // Assume empty composite types for simplicity
            },
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Table(struct_table);
        let type_struct: TypeStruct = struct_type.into();

        assert_eq!(type_struct.name, "TestTable");
    }

    #[test]
    fn test_struct_type_with_empty_params() {
        let struct_params = StructParams {
            name: "Test",
            params: &[], // Empty parameters
            default_schema: "public",
            schemas: &[],                 // Assume empty schemas for simplicity
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Params(struct_params);
        let type_struct: TypeStruct = struct_type.into();

        assert_eq!(type_struct.name, "TestParams");
        assert!(type_struct.fields.is_empty()); // No fields for empty parameters
    }

    #[test]
    fn test_struct_type_with_empty_row() {
        let struct_row = StructRow {
            name: "Test",
            columns: &[], // Empty columns
            default_schema: "public",
            schemas: &[],                 // Assume empty schemas for simplicity
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Row(struct_row);
        let type_struct: TypeStruct = struct_type.into();

        assert_eq!(type_struct.name, "TestRow");
        assert!(type_struct.fields.is_empty()); // No fields for empty row
    }

    #[test]
    fn test_struct_type_with_empty_table() {
        let struct_table = StructTable {
            table: &plugin::Table {
                rel: Some(plugin::Identifier {
                    catalog: "catalog".to_string(),
                    schema: "public".to_string(),
                    name: "test_table".to_string(),
                }),
                columns: vec![], // Empty columns
                comment: "".to_string(),
            },
            default_schema: "public",
            schema: &plugin::Schema {
                name: "public".to_string(),
                comment: "".to_string(),
                tables: vec![],          // Empty tables
                enums: vec![],           // Empty enums
                composite_types: vec![], // Empty composite types
            },
            options: &Options::default(), // Default options
        };

        let struct_type = StructType::Table(struct_table);
        let type_struct: TypeStruct = struct_type.into();

        assert_eq!(type_struct.name, "TestTable");
        assert!(type_struct.fields.is_empty()); // No fields for empty table
    }
}
