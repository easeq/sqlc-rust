use super::{get_ident, MultiLineString};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct TypeConst {
    name: String,
    value: String,
}

impl TypeConst {
    pub fn new<S: Into<String>>(name: S, value: S) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn name(&self) -> String {
        self.name.to_case(Case::ScreamingSnake)
    }

    fn generate_code(&self) -> TokenStream {
        let ident_const = get_ident(&self.name());
        let query_text = MultiLineString(&self.value).to_token_stream();

        quote! {
            pub(crate) const #ident_const: &str = #query_text;
        }
    }
}

impl ToTokens for TypeConst {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code());
    }
}

impl From<&crate::plugin::Query> for TypeConst {
    fn from(query: &crate::plugin::Query) -> Self {
        TypeConst::new(&query.name, &query.text)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn create_const(name: Option<&str>, value: Option<&str>) -> TypeConst {
        TypeConst::new(name.unwrap_or("constName"), value.unwrap_or("constValue"))
    }

    #[test]
    fn test_type_const() {
        assert_eq!(
            create_const(None, None),
            TypeConst {
                name: "constName".to_string(),
                value: "constValue".to_string()
            }
        );
        assert_eq!(
            TypeConst::new("constName".to_string(), "constValue1".to_string()),
            TypeConst {
                name: "constName".to_string(),
                value: "constValue1".to_string()
            }
        );
    }

    #[test]
    fn test_type_const_name() {
        for name in &["constName", "ConstName", "CONST_NAME", "const_name"] {
            assert_eq!(create_const(Some(name), None).name(), "CONST_NAME");
        }
    }

    #[test]
    fn test_generate_code_single_line() {
        assert_eq!(
            create_const(None, Some("Single line value"))
                .generate_code()
                .to_string(),
            quote! {
                const CONST_NAME: &str = r#"Single line value"#;
            }
            .to_string()
        );
    }

    #[test]
    fn test_generate_code_multi_line() {
        assert_eq!(
            create_const(None, Some("First line\nSecond line"))
                .generate_code()
                .to_string(),
            quote! {
                const CONST_NAME: &str = r#"
First line
Second line
"#;
            }
            .to_string()
        );
    }
}
