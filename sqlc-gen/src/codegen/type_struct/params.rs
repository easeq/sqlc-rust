use super::{Struct, StructField};
use crate::codegen::plugin;
use convert_case::{Case, Casing};
use std::collections::HashMap;

/// Represents a struct containing parameters, which could be used for code generation.
pub(crate) struct StructParams<'a> {
    /// Name of the struct (typically the name of the entity, e.g., "user")
    pub name: &'a str,

    /// List of parameters associated with the struct
    pub params: &'a [plugin::Parameter],

    /// Default schema for resolving the table schema
    pub default_schema: &'a str,

    /// A list of available schemas
    pub schemas: &'a [plugin::Schema],

    /// Options for code generation
    pub options: &'a crate::codegen::Options,
}

impl<'a> Struct<'a> for StructParams<'a> {
    /// Returns the options for code generation.
    fn options(&self) -> &'a crate::codegen::Options {
        self.options
    }

    /// Generates the name of the struct, formatted in PascalCase and appended with "Params".
    ///
    /// # Example:
    /// ```rust
    /// let struct_params = StructParams { name: "user", .. };
    /// assert_eq!(struct_params.name(), "UserParams");
    /// ```
    fn name(&self) -> String {
        format!("{}Params", self.name).to_case(Case::Pascal)
    }

    /// Generates a list of fields corresponding to the struct's parameters.
    ///
    /// This method converts the parameters into `StructField` instances, using the provided
    /// schema, options, and default schema. If a field name is duplicated or empty,
    /// a numeric suffix is added to make it unique.
    fn fields(&self) -> Vec<StructField> {
        let mut name_counts: HashMap<String, usize> = HashMap::new();

        self.params
            .iter()
            .filter_map(|param| {
                let mut field = self.create_struct_field(param)?;

                // Increment count if the name has been seen before
                let count = name_counts.entry(field.name.clone()).or_insert(0);
                if *count > 0 || field.name.is_empty() {
                    let base_name = if field.name.is_empty() {
                        "field".to_string()
                    } else {
                        field.name.clone()
                    };
                    field.name = format!("{}_{}", base_name, *count + 1);
                }
                *count += 1;

                Some(field)
            })
            .collect()
    }
}

impl<'a> StructParams<'a> {
    /// Creates a `StructField` from a parameter if the parameter contains a valid column.
    fn create_struct_field(&self, param: &plugin::Parameter) -> Option<StructField> {
        param.column.as_ref().map(|col| {
            StructField::from(
                col,                 // column
                param.number,        // position of the parameter
                self.schemas,        // schemas
                self.default_schema, // default schema
                &self.name(),        // struct name
                self.options,        // codegen options
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::plugin;
    use crate::codegen::Options;

    // Mocking plugin::Column for tests
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

    // Mocking plugin::Parameter for tests
    fn mock_plugin_parameter(column: Option<plugin::Column>, number: i32) -> plugin::Parameter {
        plugin::Parameter { column, number }
    }

    #[test]
    fn test_struct_params_name() {
        let columns = vec![mock_plugin_column("id"), mock_plugin_column("name")];
        let params = vec![
            mock_plugin_parameter(Some(columns[0].clone()), 1),
            mock_plugin_parameter(Some(columns[1].clone()), 2),
        ];
        let schemas = vec![];
        let options = Options::default();

        let struct_params = StructParams {
            name: "user",
            params: &params,
            default_schema: "public",
            schemas: &schemas,
            options: &options,
        };

        // Test if the name is properly converted to PascalCase and appended with "Params"
        assert_eq!(struct_params.name(), "UserParams");
    }

    #[test]
    fn test_struct_params_fields_with_valid_columns() {
        let columns = vec![mock_plugin_column("id"), mock_plugin_column("name")];
        let params = vec![
            mock_plugin_parameter(Some(columns[0].clone()), 1),
            mock_plugin_parameter(Some(columns[1].clone()), 2),
        ];
        let schemas = vec![];
        let options = Options::default();

        let struct_params = StructParams {
            name: "user",
            params: &params,
            default_schema: "public",
            schemas: &schemas,
            options: &options,
        };

        let fields = struct_params.fields();

        // Ensure that fields are generated correctly
        assert_eq!(fields.len(), 2);

        // Check if the field names correspond to the column names
        assert_eq!(fields[0].name, "id");
        assert_eq!(fields[1].name, "name");
    }

    #[test]
    fn test_struct_params_fields_with_missing_column() {
        let columns = vec![mock_plugin_column("id"), mock_plugin_column("name")];
        let params = vec![
            mock_plugin_parameter(None, 1), // This should be skipped since the column is None
            mock_plugin_parameter(Some(columns[1].clone()), 2),
        ];
        let schemas = vec![];
        let options = Options::default();

        let struct_params = StructParams {
            name: "user",
            params: &params,
            default_schema: "public",
            schemas: &schemas,
            options: &options,
        };

        let fields = struct_params.fields();

        // Only one field should be generated because the first parameter has no column
        assert_eq!(fields.len(), 1);

        // The only valid field should correspond to the "name" column
        assert_eq!(fields[0].name, "name");
    }

    #[test]
    fn test_struct_params_empty_params() {
        let params: Vec<plugin::Parameter> = vec![];
        let schemas = vec![];
        let options = Options::default();

        let struct_params = StructParams {
            name: "user",
            params: &params,
            default_schema: "public",
            schemas: &schemas,
            options: &options,
        };

        let fields = struct_params.fields();

        // No fields should be generated if there are no parameters
        assert_eq!(fields.len(), 0);
    }
}
