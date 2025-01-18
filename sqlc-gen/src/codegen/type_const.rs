use crate::codegen::{get_ident, MultiLineString};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// A struct representing a constant value, typically for SQL query string.
///
/// This struct stores the name and value of the constant and can generate
/// the Rust code for declaring the constant in the appropriate format.
///
/// # Fields
/// - `name`: The name of the constant (usually a string identifier).
/// - `value`: The value of the constant (typically a query or a string).
#[derive(Default, Debug, PartialEq, Clone)]
pub struct TypeConst<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> TypeConst<'a> {
    /// Creates a new `TypeConst` with the given name and value.
    ///
    /// # Arguments
    /// - `name`: A reference to the name of the constant.
    /// - `value`: A reference to the value of the constant.
    ///
    /// # Returns
    /// A new instance of `TypeConst`.
    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }

    /// Converts the constant name into **screaming snake case**.
    ///
    /// This ensures that the constant name is formatted as uppercase with
    /// underscores separating words, e.g., `"MY_CONSTANT"`.
    ///
    /// # Returns
    /// The name of the constant in screaming snake case.
    pub fn name(&self) -> String {
        self.name.to_case(Case::ScreamingSnake)
    }

    /// Generates the code for declaring the constant.
    ///
    /// This method generates the Rust code for declaring a constant in the
    /// following format:
    ///
    /// ```rust
    /// pub(crate) const CONSTANT_NAME: &str = "query_text";
    /// ```
    ///
    /// # Returns
    /// A `TokenStream` containing the generated code.
    fn generate_code(&self) -> TokenStream {
        let ident_const = get_ident(&self.name());
        let query_text = MultiLineString(self.value).to_token_stream();

        quote! {
            pub(crate) const #ident_const: &str = #query_text;
        }
    }
}

impl<'a> ToTokens for TypeConst<'a> {
    /// Converts the `TypeConst` instance into a `TokenStream` for procedural macros.
    ///
    /// This method is used to generate Rust code when the `TypeConst` is used
    /// in a procedural macro.
    ///
    /// # Arguments
    /// - `tokens`: A mutable reference to the `TokenStream` that will hold
    ///   the generated code.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code());
    }
}

impl<'a> From<&'a crate::plugin::Query> for TypeConst<'a> {
    /// Converts a `Query` struct into a `TypeConst`.
    ///
    /// This is a convenient way to create a `TypeConst` from a `Query` object,
    /// which is typically used to represent a SQL query or other constant string.
    ///
    /// # Arguments
    /// - `query`: A reference to a `Query` struct containing the `name` and `text`.
    ///
    /// # Returns
    /// A new `TypeConst` with the `name` and `text` from the `Query` struct.
    fn from(query: &'a crate::plugin::Query) -> Self {
        TypeConst::new(&query.name, &query.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    // Helper to generate expected code for TypeConst
    fn generate_expected_code(name: &str, value: &str) -> String {
        let ident_const = get_ident(&name.to_case(Case::ScreamingSnake));
        let query_text = MultiLineString(value).to_token_stream();

        quote! {
            pub(crate) const #ident_const: &str = #query_text;
        }
        .to_string()
    }

    // Test creation of TypeConst with a name and value
    #[test]
    fn test_type_const_creation() {
        let const_name = "MY_CONST";
        let const_value = "SELECT * FROM users;";

        let type_const = TypeConst::new(const_name, const_value);

        assert_eq!(type_const.name(), "MY_CONST");
        assert_eq!(type_const.value, const_value);
    }

    // Test the conversion of name to Screaming Snake case
    #[test]
    fn test_name_to_screaming_snake_case() {
        let type_const = TypeConst::new("myConst", "SELECT * FROM users;");

        // Verify that name is converted to screaming snake case (MY_CONST)
        assert_eq!(type_const.name(), "MY_CONST");
    }

    // Test the generation of code for a TypeConst
    #[test]
    fn test_generate_code() {
        let const_name = "MY_CONST";
        let const_value = "SELECT * FROM users;";

        let type_const = TypeConst::new(const_name, const_value);

        // Generate expected code
        let expected_code = generate_expected_code(const_name, const_value);

        // Compare the generated code with the expected code
        assert_eq!(type_const.generate_code().to_string(), expected_code);
    }

    // // Test converting a Query to a TypeConst
    // #[test]
    // fn test_from_query() {
    //     let query = crate::plugin::Query {
    //         name: "MY_QUERY".to_string(),
    //         text: "SELECT * FROM products WHERE price > 1000;".to_string(),
    //     };
    //
    //     let type_const: TypeConst = (&query).into();
    //
    //     // Verify the name and value of the TypeConst
    //     assert_eq!(type_const.name(), "MY_QUERY");
    //     assert_eq!(type_const.value, query.text);
    // }
    //
    // // Test generating correct code for a constant from Query
    // #[test]
    // fn test_generate_code_from_query() {
    //     let query = crate::plugin::Query {
    //         name: "MY_QUERY".to_string(),
    //         text: "SELECT * FROM products WHERE price > 1000;".to_string(),
    //     };
    //
    //     let type_const: TypeConst = (&query).into();
    //
    //     let expected_code =
    //         generate_expected_code("MY_QUERY", "SELECT * FROM products WHERE price > 1000;");
    //
    //     assert_eq!(type_const.generate_code().to_string(), expected_code);
    // }

    // Test for empty value in TypeConst
    #[test]
    fn test_empty_value_in_type_const() {
        let const_name = "EMPTY_CONST";
        let const_value = "";

        let type_const = TypeConst::new(const_name, const_value);

        assert_eq!(type_const.name(), "EMPTY_CONST");
        assert_eq!(type_const.value, const_value);

        let expected_code = generate_expected_code(const_name, const_value);

        assert_eq!(type_const.generate_code().to_string(), expected_code);
    }
}
