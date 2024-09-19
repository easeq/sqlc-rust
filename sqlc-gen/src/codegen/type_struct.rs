use super::PgDataType;
use super::{get_ident, CodePartial};
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

    pub fn name(&self) -> String {
        if !self.name.is_empty() {
            self.name.to_case(Case::Snake)
        } else {
            format!("_{}", self.number)
        }
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

#[derive(Default, Debug, Clone)]
pub enum StructType {
    #[default]
    Params,
    Row,
    Queries,
    Other {
        prefix: String,
        suffix: String,
    },
}

#[derive(Default, Debug, Clone)]
pub struct TypeStruct {
    name: String,
    struct_type: StructType,
    pub fields: Vec<StructField>,
}

impl TypeStruct {
    pub fn new<S: Into<String>>(
        name: S,
        struct_type: StructType,
        fields: Vec<StructField>,
    ) -> Self {
        Self {
            name: name.into(),
            struct_type,
            fields,
        }
    }

    pub fn exists(&self) -> bool {
        self.fields.len() > 0
    }

    pub fn get_as_arg(&self, arg_name: Option<&str>) -> TokenStream {
        let mut params_arg = quote! {};
        if let Some(name) = arg_name {
            if self.exists() {
                let ident_name = get_ident(name);
                let ident_params = get_ident(&self.name());
                params_arg = quote! {
                    #ident_name: #ident_params
                }
            }
        }

        params_arg
    }

    pub fn name(&self) -> String {
        let name = match &self.struct_type {
            StructType::Params => format!("{}Params", self.name),
            StructType::Row => format!("{}Row", self.name),
            StructType::Queries => self.name.clone(),
            StructType::Other { prefix, suffix } => {
                format!("{}{}{}", prefix, self.name.to_case(Case::Pascal), suffix)
            }
        };

        name.to_case(Case::Pascal)
    }

    pub fn generate_fields_list(&self) -> TokenStream {
        if self.fields.len() == 0 {
            quote! {}
        } else {
            let fields = self
                .fields
                .clone()
                .into_iter()
                .map(|field| {
                    let ident_field_name = get_ident(&field.name());
                    quote! { &params.#ident_field_name }
                })
                .collect::<Vec<_>>();

            quote! { #(#fields),* }
        }
    }

    fn generate_struct_field_code(&self, field: StructField) -> TokenStream {
        let field_name_ident = get_ident(&field.name());
        let field_type_ident = field.data_type();

        quote! {
            pub #field_name_ident: #field_type_ident
        }
    }
}

impl CodePartial for TypeStruct {
    fn of_type(&self) -> String {
        "struct".to_string()
    }

    fn generate_code(&self) -> TokenStream {
        if self.fields.len() == 0 {
            quote! {}
        } else {
            let ident_struct = get_ident(&self.name());
            let fields = self
                .fields
                .clone()
                .into_iter()
                .map(|field| self.generate_struct_field_code(field))
                .collect::<Vec<_>>();

            quote! {
                #[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
                pub(crate) struct #ident_struct {
                    #(#fields),*
                }
            }
        }
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
            data_type.unwrap_or(PgDataType("pg_catalog.int4".to_string())),
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
        assert_eq!(
            create_type_struct(None, Some(StructType::Queries), None).name(),
            "StructName".to_string()
        );
        assert_eq!(
            create_type_struct(
                None,
                Some(StructType::Other {
                    prefix: "A".to_string(),
                    suffix: "B".to_string()
                }),
                None
            )
            .name(),
            "AStructNameB".to_string()
        );
    }

    #[test]
    fn test_generate_field_code() {
        let type_struct = create_type_struct(None, None, None);
        assert_eq!(
            type_struct
                .generate_struct_field_code(type_struct.fields[0].clone())
                .to_string(),
            quote! {
                pub(crate) f_1:  Option<i32>
            }
            .to_string()
        )
    }

    #[test]
    fn test_generate_fields_list() {
        let type_struct = create_type_struct(None, None, None);
        assert_eq!(
            type_struct.generate_fields_list().to_string(),
            quote! {
                &params.f_1, &params.f_2, &params.f, &params.f_3, &params._3
            }
            .to_string()
        )
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
