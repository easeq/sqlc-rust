use super::{Struct, StructField};
use crate::codegen::plugin;
use convert_case::{Case, Casing};

/// A struct representing a table for which a corresponding struct will be generated.
pub(crate) struct StructTable<'a> {
    /// The table associated with this struct.
    pub table: &'a plugin::Table,

    /// The default schema used for table lookup.
    pub default_schema: &'a str,

    /// The schema that defines the table.
    pub schema: &'a plugin::Schema,

    /// Code generation options.
    pub options: &'a crate::codegen::Options,
}

impl<'a> Struct<'a> for StructTable<'a> {
    /// Returns the options associated with the struct generation.
    fn options(&self) -> &'a crate::codegen::Options {
        self.options
    }

    /// Generates the struct name based on the table name, adding the schema name if necessary,
    /// and singularizing it.
    fn name(&self) -> String {
        let table_name = self.build_table_name();
        pluralizer::pluralize(&table_name, 1, false).to_case(Case::Pascal)
    }

    /// Generates a vector of struct fields from the columns of the table.
    fn fields(&self) -> Vec<StructField> {
        self.table
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                StructField::from(
                    col,
                    i as i32,
                    &[self.schema.clone()],
                    self.default_schema,
                    &self.name(),
                    self.options,
                )
            })
            .collect()
    }

    /// Returns the table identifier with schema and catalog information.
    fn table_identifier(&self) -> Option<plugin::Identifier> {
        let table_rel = self.table.rel.as_ref()?;

        Some(plugin::Identifier {
            catalog: "".to_string(),
            schema: self.schema.name.clone(),
            name: table_rel.name.clone(),
        })
    }
}

impl<'a> StructTable<'a> {
    /// Builds the table name, prefixing it with the schema name if it's different from the default schema.
    ///
    /// This method combines the schema and table names, allowing for a fully qualified table name
    /// if the schema is not the default one.
    fn build_table_name(&self) -> String {
        let table_name = self
            .table
            .rel
            .as_ref()
            .map(|rel| rel.name.clone())
            .unwrap_or_else(|| "unknown_table".to_string());

        if self.schema.name == self.default_schema {
            table_name
        } else {
            format!("{}_{table_name}", self.schema.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::plugin;
    use crate::codegen::Options;

    // Mock structs to simulate plugin::Table, plugin::Column, plugin::Schema, and plugin::Identifier
    fn mock_column(name: &str, _number: i32) -> plugin::Column {
        plugin::Column {
            name: name.to_string(),
            not_null: false,
            is_array: false,
            comment: String::new(),
            length: 0,
            is_named_param: false,
            is_func_call: false,
            scope: String::new(),
            table: None,
            table_alias: String::new(),
            r#type: None,
            is_sqlc_slice: false,
            embed_table: None,
            original_name: String::new(),
            unsigned: false,
            array_dims: 0,
        }
    }

    fn mock_identifier(name: &str) -> plugin::Identifier {
        plugin::Identifier {
            catalog: "".to_string(),
            schema: "public".to_string(),
            name: name.to_string(),
        }
    }

    fn mock_schema(name: &str, tables: Vec<plugin::Table>) -> plugin::Schema {
        plugin::Schema {
            comment: String::new(),
            name: name.to_string(),
            tables,
            enums: Vec::new(),
            composite_types: Vec::new(),
        }
    }

    fn mock_table(name: &str, columns: Vec<plugin::Column>) -> plugin::Table {
        plugin::Table {
            rel: Some(mock_identifier(name)),
            columns,
            comment: String::new(),
        }
    }

    #[test]
    fn test_struct_table_name_with_default_schema() {
        let columns = vec![mock_column("id", 0), mock_column("name", 1)];
        let table = mock_table("user", columns);
        let schema = mock_schema("public", vec![table.clone()]);
        let default_schema = "public";
        let options = Options::default();

        let struct_table = StructTable {
            table: &table,
            default_schema,
            schema: &schema,
            options: &options,
        };

        // Test when schema is default
        assert_eq!(struct_table.name(), "User");
    }

    #[test]
    fn test_struct_table_name_with_custom_schema() {
        let columns = vec![mock_column("id", 0), mock_column("name", 1)];
        let table = mock_table("user", columns);
        let schema = mock_schema("custom", vec![table.clone()]);
        let default_schema = "public";
        let options = Options::default();

        let struct_table = StructTable {
            table: &table,
            default_schema,
            schema: &schema,
            options: &options,
        };

        // Test when schema is custom and not the default
        assert_eq!(struct_table.name(), "CustomUser");
    }

    #[test]
    fn test_struct_fields_generation() {
        let columns = vec![mock_column("id", 0), mock_column("name", 1)];
        let table = mock_table("user", columns);
        let schema = mock_schema("public", vec![table.clone()]);
        let default_schema = "public";
        let options = Options::default();

        let struct_table = StructTable {
            table: &table,
            default_schema,
            schema: &schema,
            options: &options,
        };

        let fields = struct_table.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), "id");
        assert_eq!(fields[1].name(), "name");
    }

    #[test]
    fn test_table_identifier() {
        let columns = vec![mock_column("id", 0)];
        let table = mock_table("user", columns);
        let schema = mock_schema("public", vec![table.clone()]);
        let default_schema = "public";
        let options = Options::default();

        let struct_table = StructTable {
            table: &table,
            default_schema,
            schema: &schema,
            options: &options,
        };

        let identifier = struct_table.table_identifier();
        assert!(identifier.is_some());
        let identifier = identifier.unwrap();
        assert_eq!(identifier.catalog, "");
        assert_eq!(identifier.schema, "public");
        assert_eq!(identifier.name, "user");
    }
}
