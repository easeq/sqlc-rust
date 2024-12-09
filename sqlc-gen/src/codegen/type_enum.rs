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

fn generate_enum_variant(i: usize, val: &str, seen: &mut HashSet<String>) -> TokenStream {
    let mut value = val.chars().filter_map(enum_replacer).collect::<String>();
    if seen.get(&value).is_some() || value.is_empty() {
        value = format!("value_{}", i);
    }
    let ident_variant = get_ident(&value.to_case(Case::Pascal));
    seen.insert(value);
    quote! {
        #[postgres(name=#val)]
        #[cfg_attr(feature = "serde_support", serde(rename=#val))]
        #ident_variant
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct TypeEnum {
    name: String,
    values: Vec<String>,
    derive: Vec<String>,
    attrs: Vec<String>,
}

impl TypeEnum {
    pub fn new<S: Into<String>>(name: S, values: Vec<String>) -> Self {
        Self {
            name: name.into(),
            values,
            ..Self::default()
        }
    }

    pub(crate) fn from(
        e: &crate::plugin::Enum,
        schema_name: &str,
        default_schema: &str,
        options: &crate::codegen::Options,
    ) -> Self {
        let enum_name = enum_name(&e.name, schema_name, default_schema);
        let mut e = Self::new(enum_name, e.vals.clone());
        if let Some(rules) = options.rules.as_ref() {
            e.derive = rules.derive_for(e.name(), RuleType::Enums);
            e.attrs = rules.container_attrs_for(e.name(), RuleType::Enums);
        }
        e
    }

    pub(crate) fn name(&self) -> String {
        self.name.to_case(Case::Pascal)
    }

    fn generate_code(&self) -> TokenStream {
        let ident_enum_name = get_ident(&self.name());
        let type_name = self.name().to_case(Case::Snake);
        let mut seen = HashSet::new();
        let variants = self
            .values
            .iter()
            .enumerate()
            .map(|(i, val)| generate_enum_variant(i, val, &mut seen))
            .collect::<Vec<_>>();

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
