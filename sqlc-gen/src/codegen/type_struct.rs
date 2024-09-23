use super::column_name;
use super::get_ident;
use super::plugin;
use super::PgDataType;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub is_array: bool,
    pub not_null: bool,
    pub number: i32,
    pub data_type: PgDataType,
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
        }
    }

    pub fn from(
        col: plugin::Column,
        pos: i32,
        schemas: Vec<plugin::Schema>,
        default_schema: String,
    ) -> Self {
        Self::new(
            col.name,
            pos,
            PgDataType::from(&col.r#type.unwrap().name, schemas, default_schema),
            col.is_array,
            col.not_null,
        )
    }

    pub fn name(&self) -> String {
        column_name(self.name.clone(), self.number)
    }

    pub fn data_type(&self) -> TokenStream {
        let mut tokens = self.data_type.to_token_stream();

        if self.is_array {
            let vec_ident = get_ident("Vec");
            tokens = quote! {
                #vec_ident<#tokens>
            };
        }

        if !self.not_null {
            let option_ident = get_ident("Option");
            tokens = quote! {
                #option_ident<#tokens>
            };
        }

        tokens
    }
}

impl ToTokens for StructField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_name_ident = get_ident(&self.name());
        let field_type_ident = self.data_type();

        tokens.extend(quote! {
            pub #field_name_ident: #field_type_ident
        })
    }
}

#[derive(Default, Debug, Clone)]
pub enum StructType {
    #[default]
    Default,
    Params,
    Row,
}

#[derive(Default, Debug, Clone)]
pub struct TypeStruct {
    name: String,
    pub table: Option<plugin::Identifier>,
    struct_type: StructType,
    pub fields: Vec<StructField>,
}

impl TypeStruct {
    pub fn new<S: Into<String>>(
        name: S,
        table: Option<plugin::Identifier>,
        struct_type: StructType,
        fields: Vec<StructField>,
    ) -> Self {
        Self {
            name: name.into(),
            table,
            struct_type,
            fields,
        }
    }

    pub fn name(&self) -> String {
        let name = match &self.struct_type {
            StructType::Default => format!("{}", self.name),
            StructType::Params => format!("{}Params", self.name),
            StructType::Row => format!("{}Row", self.name),
        };

        name.to_case(Case::Pascal)
    }

    fn generate_code(&self) -> TokenStream {
        if self.fields.len() == 0 {
            quote! {}
        } else {
            let ident_struct = get_ident(&self.name());
            let fields = self.fields.clone().into_iter().collect::<Vec<_>>();

            quote! {
                #[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
                pub(crate) struct #ident_struct {
                    #(#fields),*
                }
            }
        }
    }
}

impl ToTokens for TypeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_code());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_struct_field(
        name: Option<&str>,
        number: Option<i32>,
        data_type: Option<PgDataType>,
        is_array: Option<bool>,
        not_null: Option<bool>,
    ) -> StructField {
        StructField::new(
            name.unwrap_or(""),
            number.unwrap_or(0),
            data_type.unwrap_or(PgDataType::from("pg_catalog.int4", vec![], "".to_string())),
            is_array.unwrap_or_default(),
            not_null.unwrap_or_default(),
        )
    }

    #[test]
    fn test_struct_field_name() {
        assert_eq!(
            create_struct_field(Some("field1"), None, None, None, None).name(),
            "field_1".to_string()
        );
        for name in &["field_name1", "FieldName1", "Field_Name1", "FIELD_NAME1"] {
            assert_eq!(
                create_struct_field(Some(name), None, None, None, None).name(),
                "field_name_1".to_string()
            );
        }
        assert_eq!(
            create_struct_field(None, Some(2), None, None, None).name(),
            "_2".to_string()
        );
    }

    #[test]
    fn test_struct_data_type() {
        assert_eq!(
            create_struct_field(Some("field1"), None, None, None, None)
                .data_type()
                .to_string(),
            quote! { Option<i32> }.to_string()
        );
        assert_eq!(
            create_struct_field(Some("field1"), None, None, Some(true), None)
                .data_type()
                .to_string(),
            quote! { Option<Vec<i32> > }.to_string()
        );
        assert_eq!(
            create_struct_field(Some("field1"), None, None, Some(true), Some(true))
                .data_type()
                .to_string(),
            quote! { Vec<i32> }.to_string()
        );
        assert_eq!(
            create_struct_field(Some("field1"), None, None, None, Some(true))
                .data_type()
                .to_string(),
            quote! { i32 }.to_string()
        );
    }

    fn create_type_struct(
        name: Option<&str>,
        struct_type: Option<StructType>,
        fields: Option<Vec<StructField>>,
    ) -> TypeStruct {
        let default_fields = vec![
            create_struct_field(Some("f1"), None, None, None, None),
            create_struct_field(Some("F2"), None, None, None, Some(true)),
            create_struct_field(Some("f"), None, None, Some(true), None),
            create_struct_field(Some("f3"), None, None, Some(true), Some(true)),
            create_struct_field(None, Some(3), None, Some(true), Some(true)),
        ];

        TypeStruct::new(
            name.unwrap_or("struct_name"),
            Some(plugin::Identifier {
                catalog: "".to_string(),
                schema: "".to_string(),
                name: "".to_string(),
            }),
            struct_type.unwrap_or_default(),
            fields.unwrap_or(default_fields),
        )
    }

    #[test]
    fn test_struct_type() {
        assert_eq!(
            create_type_struct(None, Some(StructType::Params), None).name(),
            "StructNameParams".to_string()
        );
        assert_eq!(
            create_type_struct(None, Some(StructType::Row), None).name(),
            "StructNameRow".to_string()
        );
    }

    #[test]
    fn test_generate_code() {
        let type_struct = create_type_struct(None, None, None);
        assert_eq!(
            type_struct.generate_code().to_string(),
            quote! {
                #[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
                pub(crate) struct StructNameParams {
                    pub(crate) f_1:  Option<i32>,
                    pub(crate) f_2: i32,
                    pub(crate) f: Option<Vec<i32> >,
                    pub(crate) f_3: Vec<i32>,
                    pub(crate) _3: Vec<i32>
                }
            }
            .to_string()
        )
    }
}
