use super::{get_ident, MultiLineString};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Default, Clone)]
pub struct TypeConst {
    name: String,
    value: String,
}

impl TypeConst {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> String {
        self.name.to_case(Case::ScreamingSnake)
    }

    pub fn generate_code(&self) -> TokenStream {
        let ident_const = get_ident(&self.name());
        let query_text = MultiLineString(self.value.clone()).to_token_stream();

        quote! {
            const #ident_const: &str = #query_text;
        }
    }
}
