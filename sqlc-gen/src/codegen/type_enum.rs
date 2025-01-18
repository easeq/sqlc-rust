use std::{char, collections::HashSet};

use crate::codegen::get_ident;
use crate::codegen::options::RuleType;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

/// Generates a Pascal-case enum name based on the schema name and the default schema.
/// If the schema name is different from the default schema, it prefixes the name with the schema name.
///
/// # Arguments
/// - `name`: The original name of the enum.
/// - `schema_name`: The name of the schema.
/// - `default_schema`: The default schema name.
///
/// # Returns
/// - A Pascal-case formatted string representing the enum name.
pub(crate) fn enum_name(name: &str, schema_name: &str, default_schema: &str) -> String {
    match schema_name == default_schema {
        true => name.to_string(),
        false => format!("{schema_name}_{name}"),
    }
    .to_case(Case::Pascal)
}

/// Helper function used to replace special characters (`-`, `/`, `:`, `_`) in the enum value names with underscores (`_`),
/// while allowing alphanumeric characters to pass through unchanged.
///
/// # Arguments
/// - `c`: The character to be replaced.
///
/// # Returns
/// - `Some('_')` if the character is a special character, otherwise returns `Some(c)` if the character is alphanumeric.
fn enum_replacer(c: char) -> Option<char> {
    if ['-', '/', ':', '_'].contains(&c) {
        Some('_')
    } else if c.is_alphanumeric() {
        Some(c)
    } else {
        None
    }
}

/// A struct representing a variant of an enum.
/// It holds the original name, the formatted name, and a list of attributes.
///
/// # Fields
/// - `orig_name`: The original name of the variant (before formatting).
/// - `name`: The formatted name (e.g., Pascal case).
/// - `attrs`: A list of attributes to be applied to the variant.
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Variant {
    orig_name: String,
    name: String,
    attrs: Vec<String>,
}

impl Variant {
    /// Creates a new `Variant` instance with a given original name and formatted name.
    pub fn new<S: Into<String>>(orig_name: S, name: S) -> Self {
        Self {
            orig_name: orig_name.into(),
            name: name.into(),
            ..Self::default()
        }
    }

    /// Constructs a `Variant` from a name and options.
    pub(crate) fn from<S: Into<String>>(
        orig_name: S,
        name: S,
        enum_name: S,
        options: &crate::codegen::Options,
    ) -> Self {
        let mut v = Self::new(orig_name, name);
        if let Some(rules) = options.rules.as_ref() {
            v.attrs = rules.child_attr_for(v.name.clone(), enum_name.into(), RuleType::Enums);
        }
        v
    }

    /// Generates the code for the variant, including attributes and the variant name.
    fn generate_code(&self) -> TokenStream {
        let orig_name = &self.orig_name;
        let ident_variant = get_ident(&self.name.to_case(Case::Pascal));
        let attrs_tokens = crate::codegen::list_tokenstream(&self.attrs);
        quote! {
            #[postgres(name=#orig_name)]
            #(#attrs_tokens)*
            #ident_variant
        }
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code().to_token_stream());
    }
}

/// A struct representing an enum type with a name and a list of variants.
///
/// # Fields
/// - `name`: The name of the enum.
/// - `variants`: A list of `Variant` structs representing the variants of the enum.
/// - `derive`: A list of derive attributes for the enum.
/// - `attrs`: A list of attributes for the enum.
#[derive(Default, Debug, PartialEq)]
pub struct TypeEnum {
    pub name: String,
    pub variants: Vec<Variant>,
    derive: Vec<String>,
    attrs: Vec<String>,
}

impl TypeEnum {
    /// Creates a new `TypeEnum` with a given name and list of variants.
    pub fn new<S: Into<String>>(name: S, variants: Vec<Variant>) -> Self {
        Self {
            name: name.into(),
            variants,
            ..Self::default()
        }
    }

    /// Creates a `TypeEnum` from a plugin enum and schema information.
    pub(crate) fn from(
        e: &crate::plugin::Enum,
        schema_name: &str,
        default_schema: &str,
        options: &crate::codegen::Options,
    ) -> Self {
        let mut seen = HashSet::new();
        let enum_name = enum_name(&e.name, schema_name, default_schema);
        let mut type_enum = Self::new(
            &enum_name,
            e.vals
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    let mut name = value.chars().filter_map(enum_replacer).collect::<String>();

                    if seen.get(&name).is_some() || name.is_empty() {
                        name = format!("value_{}", i);
                    }
                    seen.insert(name.clone());
                    Variant::from(value.clone(), name, enum_name.clone(), options)
                })
                .collect::<Vec<_>>(),
        );
        if let Some(rules) = options.rules.as_ref() {
            type_enum.derive = rules.derive_for(type_enum.name(), RuleType::Enums);
            type_enum.attrs = rules.container_attrs_for(type_enum.name(), RuleType::Enums);
        }
        type_enum
    }

    /// Returns the formatted name of the enum in Pascal case.
    pub(crate) fn name(&self) -> String {
        self.name.to_case(Case::Pascal)
    }

    /// Generates the code for the enum, including derive attributes and variants.
    fn generate_code(&self) -> TokenStream {
        let ident_enum_name = get_ident(&self.name());
        let type_name = self.name().to_case(Case::Snake);
        let variants = &self.variants;

        let derive_tokens = crate::codegen::list_tokenstream(&self.derive);
        let attr_tokens = crate::codegen::list_tokenstream(&self.attrs);

        quote! {
            #[derive(postgres_derive::ToSql, postgres_derive::FromSql, #(#derive_tokens),*)]
            #(#attr_tokens)*
            #[postgres(name=#type_name)]
            pub enum #ident_enum_name {
                #(#variants),*
            }
        }
    }
}

impl ToTokens for TypeEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code().to_token_stream());
    }
}

impl From<&TypeEnum> for Ident {
    fn from(c: &TypeEnum) -> Self {
        get_ident(&c.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::Options;

    // Helper function to create a sample TypeEnum with variants
    fn create_enum_with_variants(name: Option<&str>, variants: Option<Vec<Variant>>) -> TypeEnum {
        let default_variants = vec![Variant::new("val1", "Val1"), Variant::new("val2", "Val2")];
        TypeEnum::new(
            name.unwrap_or("enum_name"),
            variants.unwrap_or(default_variants),
        )
    }

    // Test enum_name function
    #[test]
    fn test_enum_name() {
        // Test default schema
        assert_eq!(
            enum_name("enum_name", "default_schema", "default_schema"),
            "EnumName"
        );

        // Test with a different schema
        assert_eq!(
            enum_name("enum_name", "schema1", "default_schema"),
            "Schema1EnumName"
        );

        // Test with the default schema but different name case
        assert_eq!(
            enum_name("enum_name", "default_schema", "default_schema"),
            "EnumName"
        );
    }

    // Test that Variant generates correct code
    #[test]
    fn test_variant_generate_code() {
        let variant = Variant::new("val1", "Val1");

        // Expect correct generation of the code for the Variant
        let generated_code = variant.generate_code().to_string();
        let expected_code = quote! {
            #[postgres(name="val1")]
            Val1
        }
        .to_string();

        assert_eq!(generated_code, expected_code);
    }

    // Test TypeEnum creation with variants
    #[test]
    fn test_type_enum_creation_with_variants() {
        let variants = vec![Variant::new("val0", "Val0"), Variant::new("val1", "Val1")];

        let enum_name = "EnumName";
        let type_enum = create_enum_with_variants(Some(enum_name), Some(variants.clone()));

        assert_eq!(type_enum.name(), "EnumName");
        assert_eq!(type_enum.variants, variants);
    }

    // Test TypeEnum with default variants
    #[test]
    fn test_type_enum_with_default_variants() {
        let default_variants = vec![Variant::new("val1", "Val1"), Variant::new("val2", "Val2")];

        let type_enum = create_enum_with_variants(None, None);

        assert_eq!(type_enum.name(), "EnumName");
        assert_eq!(type_enum.variants, default_variants);
    }

    // Test if the generated code from TypeEnum is correct
    #[test]
    fn test_type_enum_generate_code() {
        let type_enum = create_enum_with_variants(None, None);

        let generated_code = type_enum.generate_code().to_string();
        let expected_code = quote! {
            #[derive(postgres_derive::ToSql, postgres_derive::FromSql ,)]
            #[postgres(name="enum_name")]
            pub enum EnumName {
                #[postgres(name="val1")]
                Val1,
                #[postgres(name="val2")]
                Val2
            }
        }
        .to_string();

        assert_eq!(generated_code, expected_code);
    }

    // Test the variant name replacement logic in enum_replacer function
    #[test]
    fn test_enum_replacer() {
        // Characters that need to be replaced with '_'
        assert_eq!(enum_replacer('-'), Some('_'));
        assert_eq!(enum_replacer('/'), Some('_'));
        assert_eq!(enum_replacer(':'), Some('_'));
        assert_eq!(enum_replacer('_'), Some('_'));

        // Alphanumeric characters should pass through unchanged
        assert_eq!(enum_replacer('a'), Some('a'));
        assert_eq!(enum_replacer('Z'), Some('Z'));
        assert_eq!(enum_replacer('1'), Some('1'));

        // Any other character should be replaced with None
        assert_eq!(enum_replacer('$'), None);
        assert_eq!(enum_replacer(' '), None);
    }

    // Test handling of enum variants with duplicate or empty names
    #[test]
    fn test_duplicate_variant_names() {
        let variants = vec![
            Variant::new("val1", "Val1"),
            Variant::new("val1", "Val1"), // Duplicate
            Variant::new("val2", "Val2"),
        ];

        let type_enum = create_enum_with_variants(None, Some(variants.clone()));

        // In case of duplicate, the second variant should get a unique name
        let expected_variants = vec![
            Variant::new("val1", "Val1"),
            Variant::new("val1", "Val1"),
            Variant::new("val2", "Val2"),
        ];

        assert_eq!(type_enum.variants, expected_variants);
    }

    // Test for handling empty variant names
    #[test]
    fn test_empty_variant_name() {
        let variants = vec![
            Variant::new("val1", "Val1"),
            Variant::new("", "Value_1"), // Empty name variant
            Variant::new("val2", "Val2"),
        ];

        let type_enum = create_enum_with_variants(None, Some(variants.clone()));

        // The empty name should be replaced with a default name like "value_1"
        let expected_variants = vec![
            Variant::new("val1", "Val1"),
            Variant::new("", "Value_1"), // Renamed to "Value_1"
            Variant::new("val2", "Val2"),
        ];

        assert_eq!(type_enum.variants, expected_variants);
    }

    // Test the enum name formatting with PascalCase
    #[test]
    fn test_enum_name_pascal_case() {
        let type_enum = create_enum_with_variants(None, None);

        // Check that the enum name is properly formatted in PascalCase
        assert_eq!(type_enum.name(), "EnumName");
    }

    // Test that derive attributes are correctly applied
    #[test]
    fn test_enum_with_derives() {
        let options = Options {
            rules: Some(crate::codegen::options::Rules(HashSet::new())),
            ..Default::default()
        };

        let e = crate::plugin::Enum {
            name: "enum_name".to_string(),
            vals: vec!["val1".to_string(), "val2".to_string()],
            ..Default::default()
        };

        let type_enum = TypeEnum::from(&e, "schema_name", "default_schema", &options);

        assert!(type_enum.derive.is_empty()); // Rules are empty, so derive should be empty
    }
}
