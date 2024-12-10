use std::{char, collections::HashSet};

use crate::codegen::get_ident;
use crate::codegen::options::RuleType;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

pub(crate) fn enum_name(name: &str, schema_name: &str, default_schema: &str) -> String {
    match schema_name == default_schema {
        true => name.to_string(),
        false => format!("{schema_name}_{name}"),
    }
    .to_case(Case::Pascal)
}

fn enum_replacer(c: char) -> Option<char> {
    if ['-', '/', ':', '_'].contains(&c) {
        Some('_')
    } else if c.is_alphanumeric() {
        Some(c)
    } else {
        None
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Variant {
    orig_name: String,
    name: String,
    attrs: Vec<String>,
}

impl Variant {
    fn new<S: Into<String>>(orig_name: S, name: S) -> Self {
        Self {
            orig_name: orig_name.into(),
            name: name.into(),
            ..Self::default()
        }
    }

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

    fn generate_code(&self) -> TokenStream {
        let orig_name = self.orig_name.clone();
        let ident_variant = get_ident(&self.name.to_case(Case::Pascal));
        let attrs_tokens = crate::codegen::list_tokenstream(&self.attrs);
        quote! {
            #[postgres(name=#orig_name)]
            // #[cfg_attr(feature = "serde_support", serde(rename=#val))]
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

#[derive(Default, Debug, PartialEq)]
pub struct TypeEnum {
    name: String,
    variants: Vec<Variant>,
    derive: Vec<String>,
    attrs: Vec<String>,
}

impl TypeEnum {
    pub fn new<S: Into<String>>(name: S, variants: Vec<Variant>) -> Self {
        Self {
            name: name.into(),
            variants,
            ..Self::default()
        }
    }

    pub(crate) fn from(
        e: &crate::plugin::Enum,
        schema_name: &str,
        default_schema: &str,
        options: &crate::codegen::Options,
    ) -> Self {
        let mut seen = HashSet::new();
        let enum_name = enum_name(&e.name, schema_name, default_schema);
        let mut type_enum = Self::new(
            enum_name.clone(),
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

    pub(crate) fn name(&self) -> String {
        self.name.to_case(Case::Pascal)
    }

    fn generate_code(&self) -> TokenStream {
        let ident_enum_name = get_ident(&self.name());
        let type_name = self.name().to_case(Case::Snake);
        let variants = &self.variants;

        let derive_tokens = crate::codegen::list_tokenstream(&self.derive);
        let attr_tokens = crate::codegen::list_tokenstream(&self.attrs);

        quote! {
            #[derive(postgres_derive::ToSql, postgres_derive::FromSql)]
            #[derive(#(#derive_tokens),*)]
            #(#attr_tokens)*
            // #[derive(Clone, Debug, PartialEq, postgres_derive::ToSql, postgres_derive::FromSql)]
            // #[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
            // #[cfg_attr(feature = "hash", derive(Eq, Hash))]
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

    fn create_enum(name: Option<&str>, values: Option<Vec<String>>) -> TypeEnum {
        let default_values = vec!["val1".to_string(), "val2".to_string()];
        TypeEnum::new(
            name.unwrap_or("enum_name"),
            values.unwrap_or(default_values),
        )
    }

    #[test]
    fn test_type_enum() {
        let values = vec!["val0".to_string(), "val1".to_string()];
        assert_eq!(
            create_enum(Some("ENUM_NAME"), Some(values.clone())),
            TypeEnum {
                name: "ENUM_NAME".to_string(),
                values: values.clone(),
            }
        );
    }

    #[test]
    fn test_type_enum_name() {
        for name in &["enumName", "EnumName", "ENUM_NAME", "enum_name"] {
            assert_eq!(create_enum(Some(name), None).name(), "EnumName");
        }
    }

    #[test]
    fn test_generate_code() {
        assert_eq!(
            create_enum(None, None).generate_code().to_string(),
            quote! {
                #[derive(Clone, Debug, PartialEq, postgres_derive::ToSql, postgres_derive::FromSql)]
                pub enum EnumName {
                    #[postgres(name="val1")]
                    Val1,
                    #[postgres(name="val2")]
                    Val2
                }
            }
            .to_string()
        );
    }
}
