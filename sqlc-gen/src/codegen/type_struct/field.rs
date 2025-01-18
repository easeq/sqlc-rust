use super::{column_name, same_table};
use crate::codegen::options::RuleType;
use crate::codegen::{get_ident, plugin, PgDataType};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// A struct representing a field in a generated struct.
///
/// This struct is used to hold information about a field in a Rust struct
/// that corresponds to a database column. It includes the field's name,
/// data type, whether it is an array, whether it is nullable, and any
/// additional attributes.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct StructField {
    /// The name of the field.
    pub name: String,

    /// Indicates whether the field is an array type.
    pub is_array: bool,

    /// Indicates whether the field is non-nullable.
    pub not_null: bool,

    /// The position of the field in the original column (used for ordering).
    pub number: i32,

    /// The data type of the field.
    pub data_type: PgDataType,

    /// Optional attributes for the field (e.g., `#[serde(rename = "field_name")]`).
    pub attrs: Vec<String>,
}

impl StructField {
    /// Creates a new `StructField` instance.
    ///
    /// # Parameters
    /// - `name`: The name of the field.
    /// - `number`: The position of the field in the original column.
    /// - `data_type`: The data type of the field.
    /// - `is_array`: Whether the field is an array type.
    /// - `not_null`: Whether the field is non-nullable.
    ///
    /// # Returns
    /// A new `StructField` instance.
    pub fn new<S: Into<String>>(
        name: S,
        number: i32,
        data_type: PgDataType,
        is_array: bool,
        not_null: bool,
    ) -> Self {
        Self {
            name: name.into(),
            number,
            data_type,
            is_array,
            not_null,
            attrs: vec![], // Default to empty vector for attributes
        }
    }

    /// Constructs a `StructField` from a database column and schema information.
    ///
    /// This function transforms a `plugin::Column` from the database into
    /// a `StructField` that can be used in code generation. The column's
    /// name, type, array status, and nullability are taken into account.
    ///
    /// # Parameters
    /// - `col`: The column representing a field in the database.
    /// - `pos`: The position of the field in the original schema (used for naming).
    /// - `schemas`: A list of available schemas to check against.
    /// - `default_schema`: The default schema name.
    /// - `struct_name`: The name of the struct being generated.
    /// - `options`: Configuration options, including rules for field attributes.
    ///
    /// # Returns
    /// A new `StructField` instance constructed from the column.
    pub fn from(
        col: &plugin::Column,
        pos: i32,
        schemas: &[plugin::Schema],
        default_schema: &str,
        struct_name: &str,
        options: &crate::codegen::Options,
    ) -> Self {
        let mut sf = Self::new(
            col.name.clone(),
            pos,
            PgDataType::from_col(col, schemas, default_schema),
            col.is_array,
            col.not_null,
        );

        // Apply attributes based on rules in options if available
        if let Some(rules) = options.rules.as_ref() {
            sf.attrs = rules.child_attr_for(sf.name(), struct_name.to_string(), RuleType::Structs);
        }

        sf
    }

    /// Checks if this `StructField` matches a given column from the database.
    ///
    /// This method compares the field's name, type, and table against a column
    /// from the database to determine if they are compatible. This is useful for
    /// matching fields to database columns during code generation.
    ///
    /// # Parameters
    /// - `col`: The column to compare against.
    /// - `field_table`: The table associated with the field (optional).
    /// - `schemas`: A list of schemas to check against.
    /// - `default_schema`: The default schema to consider if no schema is explicitly set.
    /// - `pos`: The position of the column in the schema.
    ///
    /// # Returns
    /// `true` if the field matches the column in name, type, and table, otherwise `false`.
    pub(crate) fn matches_column(
        &self,
        col: &plugin::Column,
        field_table: Option<&plugin::Identifier>,
        schemas: &[plugin::Schema],
        default_schema: &str,
        pos: i32,
    ) -> bool {
        self.has_matching_name(col, pos)
            && self.has_matching_type(col, schemas, default_schema)
            && self.has_matching_table(col, field_table, default_schema)
    }

    /// Checks if the field name matches the column's name.
    ///
    /// This is used by `matches_column()` to ensure the field and column
    /// have the same name, adjusted for the field's position.
    ///
    /// # Parameters
    /// - `col`: The column to compare against.
    /// - `pos`: The position of the column in the schema.
    ///
    /// # Returns
    /// `true` if the names match, otherwise `false`.
    fn has_matching_name(&self, col: &plugin::Column, pos: i32) -> bool {
        self.name() == column_name(&col.name, pos)
    }

    /// Checks if the field type matches the column's type.
    ///
    /// This is used by `matches_column()` to verify that the field and column
    /// have the same data type.
    ///
    /// # Parameters
    /// - `col`: The column to compare against.
    /// - `schemas`: A list of schemas to check against.
    /// - `default_schema`: The default schema name.
    ///
    /// # Returns
    /// `true` if the types match, otherwise `false`.
    fn has_matching_type(
        &self,
        col: &plugin::Column,
        schemas: &[plugin::Schema],
        default_schema: &str,
    ) -> bool {
        self.data_type.to_string() == PgDataType::from_col(col, schemas, default_schema).to_string()
    }

    /// Checks if the field's table matches the column's table.
    ///
    /// This is used by `matches_column()` to verify that the column is in the same table
    /// as the field.
    ///
    /// # Parameters
    /// - `col`: The column to compare against.
    /// - `field_table`: The table associated with the field (optional).
    /// - `default_schema`: The default schema to consider.
    ///
    /// # Returns
    /// `true` if the tables match, otherwise `false`.
    fn has_matching_table(
        &self,
        col: &plugin::Column,
        field_table: Option<&plugin::Identifier>,
        default_schema: &str,
    ) -> bool {
        same_table(col.table.as_ref(), field_table, default_schema)
    }

    /// Returns the formatted column name based on the field's position.
    ///
    /// This method combines the field's base name and its position number
    /// to create a unique column name.
    ///
    /// # Returns
    /// A formatted string representing the column name.
    pub fn name(&self) -> String {
        column_name(&self.name, self.number)
    }

    /// Generates the type of the field, including adjustments for array and nullability.
    ///
    /// This method generates the type token, adjusting for whether the field is
    /// an array and whether it is nullable. The type is returned as a `TokenStream`.
    ///
    /// # Returns
    /// A `TokenStream` representing the field's type.
    fn data_type(&self) -> TokenStream {
        let mut tokens = self.data_type.to_token_stream();

        // Wrap the type in `Vec<T>` if it's an array
        if self.is_array {
            tokens = quote!(Vec<#tokens>);
        }

        // Wrap the type in `Option<T>` if it's nullable
        if !self.not_null {
            tokens = quote!(Option<#tokens>);
        }

        tokens
    }
}

impl ToTokens for StructField {
    /// Converts the `StructField` to a `TokenStream` representation for code generation.
    ///
    /// This method is used to convert the field's name, type, and any attributes into
    /// the appropriate Rust code, such as `pub field_name: field_type`.
    ///
    /// # Parameters
    /// - `tokens`: A mutable reference to a `TokenStream` that will be extended with the generated code.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_name_ident = get_ident(&self.name());
        let field_type_ident = self.data_type();
        let attrs_tokens = crate::codegen::list_tokenstream(&self.attrs);

        tokens.extend(quote! {
            #(#attrs_tokens)*
            pub #field_name_ident: #field_type_ident
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::options::Rules;
    use crate::codegen::{plugin, Options, PgDataType};

    // Helper function to create a mock plugin column
    fn mock_column(name: &str, is_array: bool, not_null: bool, pg_type: &str) -> plugin::Column {
        plugin::Column {
            name: name.to_string(),
            table: None, // No table for simplicity
            is_array,
            not_null,
            comment: String::new(),
            length: 0,
            is_named_param: false,
            is_func_call: false,
            scope: String::new(),
            table_alias: String::new(),
            r#type: Some(plugin::Identifier {
                name: pg_type.to_string(),
                ..Default::default()
            }),
            is_sqlc_slice: false,
            embed_table: None,
            original_name: name.to_string(),
            unsigned: false,
            array_dims: 0,
        }
    }

    /// Test the `new()` method of `StructField`.
    #[test]
    fn test_new() {
        let field = StructField::new(
            "my_field",
            1,
            PgDataType::from_str("int4", &[], "public"), // Using from_str
            true,                                        // is_array
            false,                                       // not_null
        );

        assert_eq!(field.name, "my_field");
        assert_eq!(field.number, 1);
        assert_eq!(field.data_type.to_string(), "i32");
        assert!(field.is_array);
        assert!(!field.not_null);
    }

    /// Test the `from()` method of `StructField`.
    #[test]
    fn test_from() {
        let col = mock_column("my_field", true, false, "int4");
        let schemas = vec![]; // Empty schemas for simplicity
        let default_schema = "public";
        let struct_name = "MyStruct";

        // Create an empty Options struct (without rules)
        let options = Options {
            rules: None, // No rules in this test
            ..Default::default()
        };

        let field = StructField::from(&col, 1, &schemas, default_schema, struct_name, &options);

        assert_eq!(field.name, "my_field");
        assert_eq!(field.number, 1);
        assert_eq!(field.data_type.to_string(), "i32");
        assert!(field.is_array);
        assert!(!field.not_null);
        assert!(field.attrs.is_empty());
    }

    /// Test the `from()` method of `StructField` with rules.
    #[test]
    fn test_from_with_rules() {
        let col = mock_column("my_field", true, false, "int4");
        let schemas = vec![]; // Empty schemas for simplicity
        let default_schema = "public";
        let struct_name = "MyStruct";

        // Simulate setting rules in Options
        let rules = Rules::default(); // Assume we have a default rule here for simplicity
        let options = Options {
            rules: Some(rules), // Set rules to Some
            ..Default::default()
        };

        let field = StructField::from(&col, 1, &schemas, default_schema, struct_name, &options);

        assert_eq!(field.name, "my_field");
        assert_eq!(field.number, 1);
        assert_eq!(field.data_type.to_string(), "i32");
        assert!(field.is_array);
        assert!(!field.not_null);
        assert!(!field.attrs.is_empty()); // Now attrs should be populated if rules are applied
    }

    /// Test `matches_column()` for exact matches.
    #[test]
    fn test_matches_column() {
        let field = StructField::new(
            "my_field",
            1,
            PgDataType::from_str("int4", &[], "public"),
            true,
            false,
        );

        let col = mock_column("my_field", true, false, "int4");
        let schemas = vec![]; // Empty schemas for simplicity
        let default_schema = "public";

        assert!(field.matches_column(
            &col,
            None, // No table
            &schemas,
            default_schema,
            1,
        ));
    }

    /// Test `matches_column()` when fields do not match.
    #[test]
    fn test_matches_column_mismatch() {
        let field = StructField::new(
            "my_field",
            1,
            PgDataType::from_str("int4", &[], "public"),
            false,
            true,
        );

        let col = mock_column("other_field", false, true, "int4");
        let schemas = vec![]; // Empty schemas for simplicity
        let default_schema = "public";

        assert!(!field.matches_column(
            &col,
            None, // No table
            &schemas,
            default_schema,
            1,
        ));
    }

    /// Test `data_type()` for both array and non-nullable fields.
    #[test]
    fn test_data_type() {
        let field = StructField::new(
            "my_field",
            1,
            PgDataType::from_str("int4", &[], "public"), // Using from_str
            true,                                        // is_array
            false,                                       // not_null
        );
        let tokens = field.data_type();
        let expected = quote!(Option<Vec<i32>>);
        assert_eq!(tokens.to_string(), expected.to_string());

        let field_non_array = StructField::new(
            "my_field",
            1,
            PgDataType::from_str("int4", &[], "public"), // Using from_str
            false,                                       // not an array
            false,                                       // not_nullable
        );
        let tokens_non_array = field_non_array.data_type();
        let expected_non_array = quote!(Option<i32>);
        assert_eq!(tokens_non_array.to_string(), expected_non_array.to_string());

        let field_not_nullable = StructField::new(
            "my_field",
            1,
            PgDataType::from_str("int4", &[], "public"), // Using from_str
            false,                                       // not an array
            true,                                        // nullable
        );
        let tokens_not_nullable = field_not_nullable.data_type();
        let expected_not_nullable = quote!(i32);
        assert_eq!(
            tokens_not_nullable.to_string(),
            expected_not_nullable.to_string()
        );
    }

    /// Test `to_tokens()` to ensure it generates the expected code.
    #[test]
    fn test_to_tokens() {
        let field = StructField::new(
            "my_field",
            1,
            PgDataType::from_str("int4", &[], "public"), // Using from_str
            true,                                        // is_array
            false,                                       // not_null
        );

        let mut tokens = TokenStream::new();
        field.to_tokens(&mut tokens);

        let expected_tokens = quote! {
            pub my_field: Option<Vec<i32>>
        };

        assert_eq!(tokens.to_string(), expected_tokens.to_string());
    }

    // /// Test `get_attrs_tokens()` when attributes are `Some` or `None`.
    // #[test]
    // fn test_get_attrs_tokens() {
    //     // Test with attributes
    //     let field_with_attrs = StructField {
    //         attrs: vec!["#[serde(rename = \"my_field\")]".to_string()],
    //         ..StructField::new(
    //             "my_field",
    //             1,
    //             PgDataType::from_str("int4", &[], "public"),
    //             false,
    //             false,
    //         )
    //     };
    //
    //     let tokens_with_attrs = crate::codegen::list_tokenstream(&field_with_attrs.attrs);
    //     let expected_attrs = quote!(#[serde(rename = "my_field")]);
    //     assert_eq!(tokens_with_attrs.into(), expected_attrs.to_string());
    //
    //     // Test with no attributes
    //     let field_no_attrs = StructField {
    //         attrs: vec![],
    //         ..StructField::new(
    //             "my_field",
    //             1,
    //             PgDataType::from_str("int4", &[], "public"),
    //             false,
    //             false,
    //         )
    //     };
    //
    //     let tokens_no_attrs = crate::codegen::list_tokenstream(&[]);
    //     assert_eq!(tokens_no_attrs.into(), "".to_string());
    // }
}
