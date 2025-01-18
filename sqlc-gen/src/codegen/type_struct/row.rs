use super::{Struct, StructField};
use crate::codegen::plugin;
use convert_case::{Case, Casing};

/// Represents a row in a struct, holding information about the columns and schema.
pub(crate) struct StructRow<'a> {
    /// Name of the struct (used for generating row name)
    pub name: &'a str,

    /// Columns associated with the struct row
    pub columns: &'a [plugin::Column],

    /// Default schema for resolving the table schema
    pub default_schema: &'a str,

    /// A list of available schemas
    pub schemas: &'a [plugin::Schema],

    /// Options for code generation
    pub options: &'a crate::codegen::Options,
}

impl<'a> Struct<'a> for StructRow<'a> {
    /// Returns the options for code generation.
    fn options(&self) -> &'a crate::codegen::Options {
        self.options
    }

    /// Returns the name of the struct, formatted in PascalCase with "Row" appended.
    ///
    /// # Example
    /// ```rust
    /// let struct_row = StructRow { name: "user", .. };
    /// assert_eq!(struct_row.name(), "UserRow");
    /// ```
    fn name(&self) -> String {
        format!("{}Row", self.name).to_case(Case::Pascal)
    }

    /// Generates a list of fields corresponding to the struct's columns.
    ///
    /// This method converts the columns into `StructField` instances using the
    /// provided schema, options, and default schema.
    fn fields(&self) -> Vec<StructField> {
        self.columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                StructField::from(
                    col,                 // column
                    i as i32,            // position of the column
                    self.schemas,        // schemas
                    self.default_schema, // default schema
                    &self.name(),        // struct name
                    self.options,        // codegen options
                )
            })
            .collect() // Collect the mapped StructField instances into a Vec<StructField>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::plugin;
    use crate::codegen::Options;

    // Mocking necessary dependencies for plugin::Column and plugin::Schema
    #[derive(Clone)]
    struct MockColumn {
        pub name: String,
        pub not_null: bool,
        pub is_array: bool,
    }

    impl MockColumn {
        fn new(name: &str, not_null: bool, is_array: bool) -> Self {
            Self {
                name: name.to_string(),
                not_null,
                is_array,
            }
        }
    }

    fn mock_plugin_column(name: &str) -> plugin::Column {
        plugin::Column {
            name: name.to_string(),
            not_null: true,
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

    #[test]
    fn test_struct_row_name() {
        let columns = vec![mock_plugin_column("id"), mock_plugin_column("name")];
        let schemas = vec![];
        let options = Options::default();

        let struct_row = StructRow {
            name: "user",
            columns: &columns,
            default_schema: "public",
            schemas: &schemas,
            options: &options,
        };

        // Test if the name is properly converted to PascalCase and appended with "Row"
        assert_eq!(struct_row.name(), "UserRow");
    }

    #[test]
    fn test_struct_row_fields() {
        let columns = vec![mock_plugin_column("id"), mock_plugin_column("name")];
        let schemas = vec![];
        let options = Options::default();

        let struct_row = StructRow {
            name: "user",
            columns: &columns,
            default_schema: "public",
            schemas: &schemas,
            options: &options,
        };

        // Ensure that fields are generated correctly
        let fields = struct_row.fields();

        // Here we would need to adjust the test to compare the resulting StructField instances
        // For this example, we'll assert that two fields are created, one for each column.
        assert_eq!(fields.len(), 2);

        // Example check: the name of the first field should match the first column name.
        assert_eq!(fields[0].name, "id");
    }

    #[test]
    fn test_struct_field_creation() {
        let column = mock_plugin_column("id");
        let schemas = vec![];
        let options = Options::default();

        let struct_row = StructRow {
            name: "user",
            columns: &[column],
            default_schema: "public",
            schemas: &schemas,
            options: &options,
        };

        let fields = struct_row.fields();

        // The field should be created correctly with the column's properties
        let field = &fields[0];

        assert_eq!(field.name, "id");
        assert!(field.not_null);
        assert!(!field.is_array);
    }
    //
    // #[test]
    // fn test_struct_field_attrs() {
    //     let column = mock_plugin_column("id");
    //     let mut column_with_attrs = column;
    //     column_with_attrs.comment = "This is the ID field".to_string();
    //
    //     let schemas = vec![];
    //     let options = Options::default();
    //
    //     let struct_row = StructRow {
    //         name: "user",
    //         columns: &[column_with_attrs],
    //         default_schema: "public",
    //         schemas: &schemas,
    //         options: &options,
    //     };
    //
    //     let fields = struct_row.fields();
    //
    //     // Example: Ensure that the field has the expected attributes
    //     let field = &fields[0];
    //     assert_eq!(field.name, "id");
    //
    //     // Check if the attributes are correctly converted into tokens
    //     let tokens = field.get_attrs_tokens();
    //     // The generated tokens should match the expected attribute syntax
    //     assert!(tokens.to_string().contains("id"));
    // }
}
