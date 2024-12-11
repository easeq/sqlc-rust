use super::{column_name, same_table};
use crate::codegen::options::RuleType;
use crate::codegen::{get_ident, plugin, PgDataType};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub is_array: bool,
    pub not_null: bool,
    pub number: i32,
    pub data_type: PgDataType,
    attrs: Vec<String>,
}

impl StructField {
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
            attrs: vec![],
        }
    }

    pub fn from(
        col: &plugin::Column,
        pos: i32,
        schemas: &[plugin::Schema],
        default_schema: &str,
        struct_name: &str,
        options: &crate::codegen::Options,
    ) -> Self {
        let mut sf = Self::new(
            &col.name,
            pos,
            PgDataType::from_col(col, &schemas, default_schema),
            col.is_array,
            col.not_null,
        );

        if let Some(rules) = options.rules.as_ref() {
            sf.attrs = rules.child_attr_for(sf.name(), struct_name.to_string(), RuleType::Structs);
        }

        sf
    }

    pub(crate) fn matches_column(
        &self,
        col: &crate::plugin::Column,
        field_table: Option<&plugin::Identifier>,
        schemas: &[plugin::Schema],
        default_schema: &str,
        pos: i32,
    ) -> bool {
        let same_name = self.name() == column_name(&col.name, pos);

        let same_type = self.data_type.to_string()
            == PgDataType::from_col(col, schemas, default_schema).to_string();

        let same_table = same_table(col.table.as_ref(), field_table, default_schema);

        same_name && same_type && same_table
    }

    fn name(&self) -> String {
        column_name(&self.name, self.number)
    }

    fn data_type(&self) -> TokenStream {
        let mut tokens = self.data_type.to_token_stream();

        if self.is_array {
            tokens = quote!(Vec<#tokens>);
        }

        if !self.not_null {
            tokens = quote!(Option<#tokens>);
        }

        tokens
    }

    pub(crate) fn to_pg_query_slice_item(&self, var_name: &syn::Ident) -> TokenStream {
        let ident_field_name = get_ident(&self.name());
        quote! { &#var_name.#ident_field_name }
    }
}

impl ToTokens for StructField {
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
