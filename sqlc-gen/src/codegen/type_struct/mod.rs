use crate::codegen::options::RuleType;
use crate::codegen::{plugin, DataType};
use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub(crate) use field::*;
pub(crate) use params::*;
pub(crate) use row::*;
pub(crate) use table::*;

mod field;
mod params;
mod row;
mod table;

fn column_name(name: &str, pos: i32) -> String {
    match name.is_empty() {
        false => name.to_case(convert_case::Case::Snake),
        true => format!("_{}", pos),
    }
}

fn same_table(
    col_table: Option<&plugin::Identifier>,
    struct_table: Option<&plugin::Identifier>,
    default_schema: &str,
) -> bool {
    if let Some(table_id) = col_table {
        let mut schema = table_id.schema.as_str();
        if schema.is_empty() {
            schema = default_schema;
        }

        if let Some(f) = struct_table {
            table_id.catalog == f.catalog && schema == f.schema && table_id.name == f.name
        } else {
            false
        }
    } else {
        false
    }
}

trait Struct<'a> {
    fn options(&self) -> &'a crate::codegen::Options;
    fn name(&self) -> String;
    fn fields(&self) -> Vec<StructField>;
    fn derive(&self) -> Option<Vec<String>> {
        if let Some(rules) = self.options().rules.as_ref() {
            Some(rules.derive_for(&self.name(), RuleType::Structs))
        } else {
            None
        }
    }

    fn attrs(&self) -> Option<Vec<String>> {
        if let Some(rules) = self.options().rules.as_ref() {
            Some(rules.container_attrs_for(&self.name(), RuleType::Structs))
        } else {
            None
        }
    }
    fn table_identifier(&self) -> Option<plugin::Identifier> {
        None
    }
}

pub enum StructType<'a> {
    Params(StructParams<'a>),
    Row(StructRow<'a>),
    Table(StructTable<'a>),
}

impl<'a> StructType<'a> {
    fn as_trait(&self) -> &dyn Struct {
        match self {
            Self::Params(t) => t,
            Self::Row(t) => t,
            Self::Table(t) => t,
        }
    }
}

impl<'a> From<StructType<'a>> for TypeStruct {
    fn from(struct_type: StructType<'a>) -> Self {
        let st = struct_type.as_trait();
        Self {
            name: st.name(),
            fields: st.fields(),
            table: st.table_identifier(),
            derive: st.derive().unwrap_or_default(),
            attrs: st.attrs().unwrap_or_default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TypeStruct {
    name: String,
    pub table: Option<plugin::Identifier>,
    pub fields: Vec<StructField>,
    derive: Vec<String>,
    attrs: Vec<String>,
}

impl TypeStruct {
    pub fn has_same_fields(
        &self,
        columns: &[plugin::Column],
        schemas: &[plugin::Schema],
        default_schema: &str,
    ) -> bool {
        if self.fields.len() != columns.len() {
            false
        } else {
            self.fields
                .iter()
                .zip(columns.iter())
                .enumerate()
                .all(|(i, (field, col))| {
                    field.matches_column(
                        col,
                        self.table.as_ref(),
                        schemas,
                        default_schema,
                        i as i32,
                    )
                })
        }
    }

    pub(crate) fn name(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn data_type(&self) -> DataType {
        DataType(self.name.clone())
    }

    pub(crate) fn to_pg_query_slice(&self, var_name: &syn::Ident) -> TokenStream {
        let fields = self
            .fields
            .iter()
            .map(|field| field.to_pg_query_slice_item(&var_name))
            .collect::<Vec<_>>();
        quote! { #(#fields),* }
    }

    fn generate_code(&self) -> TokenStream {
        if self.fields.len() == 0 {
            quote! {}
        } else {
            let ident_struct = self.data_type();
            let fields = self.fields.iter().collect::<Vec<_>>();
            let derive_tokens = crate::codegen::list_tokenstream(&self.derive);
            let attr_tokens = crate::codegen::list_tokenstream(&self.attrs);

            quote! {
                #[derive(sqlc_core::FromPostgresRow, #(#derive_tokens),*)]
                #(#attr_tokens)*
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
            data_type.unwrap_or(PgDataType::from("pg_catalog.int4", &[], "")),
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
                #[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
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
