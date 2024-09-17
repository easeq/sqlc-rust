use super::get_ident;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Default)]
pub struct TypeEnum {
    name: String,
    values: Vec<String>,
}

impl TypeEnum {
    pub fn new(name: String, values: Vec<String>) -> Self {
        Self { name, values }
    }

    fn name(&self) -> String {
        self.name.to_case(Case::Pascal)
    }

    pub fn generate_code(&self) -> TokenStream {
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
