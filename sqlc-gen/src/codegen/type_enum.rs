use super::{get_ident, CodePartial};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

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

    fn name(&self) -> String {
        self.name.to_case(Case::Pascal)
    }
}

impl CodePartial for TypeEnum {
    fn of_type(&self) -> String {
        "enum".to_string()
    }

    fn generate_code(&self) -> TokenStream {
        let ident_enum_name = get_ident(&self.name());
        let variants = self
            .values
            .clone()
            .into_iter()
            .map(|val| get_ident(&val.to_case(Case::Pascal)))
            .collect::<Vec<_>>();

        quote! {
            #[derive(Debug, Display)]
            pub enum #ident_enum_name {
                #(#variants),*
            }
        }
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
                #[derive(Debug, Display)]
                pub enum EnumName {
                    Val1,
                    Val2
                }
            }
            .to_string()
        );
    }
}
