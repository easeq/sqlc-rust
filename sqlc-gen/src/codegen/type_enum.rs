use std::{char, collections::HashSet};

use super::get_ident;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn enum_name(name: &str, schema_name: &str, default_schema: &str) -> String {
    match schema_name == default_schema {
        true => name.to_string(),
        false => format!("{}_{name}", schema_name),
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

fn generate_enum_variant(i: usize, val: String, seen: &mut HashSet<String>) -> TokenStream {
    let mut value = val.chars().filter_map(enum_replacer).collect::<String>();
    if seen.get(&value).is_some() || value.is_empty() {
        value = format!("value_{}", i);
    }
    seen.insert(value.clone());
    let ident_variant = get_ident(&value.to_case(Case::Pascal));
    quote! {
        #[postgres(name=#val)]
        #ident_variant
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct TypeEnum {
    name: String,
    values: Vec<String>,
}

impl TypeEnum {
    pub fn new<S: Into<String>>(name: S, values: Vec<String>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }

    pub fn name(&self) -> String {
        self.name.to_case(Case::Pascal)
    }

    fn generate_code(&self) -> TokenStream {
        let ident_enum_name = get_ident(&self.name());
        let mut seen = HashSet::new();
        let variants = self
            .values
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, val)| generate_enum_variant(i, val, &mut seen))
            .collect::<Vec<_>>();

        quote! {
            #[derive(Clone, Debug, PartialEq, postgres_derive::ToSql, postgres_derive::FromSql)]
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
