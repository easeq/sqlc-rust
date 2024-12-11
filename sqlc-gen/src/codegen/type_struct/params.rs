use super::{Struct, StructField};
use crate::codegen::plugin;
use convert_case::{Case, Casing};

pub(crate) struct StructParams<'a> {
    pub name: &'a str,
    pub params: &'a [plugin::Parameter],
    pub default_schema: &'a str,
    pub schemas: &'a [plugin::Schema],
    pub options: &'a crate::codegen::Options,
}

impl<'a> Struct<'a> for StructParams<'a> {
    fn options(&self) -> &'a crate::codegen::Options {
        self.options
    }

    fn name(&self) -> String {
        format!("{}Params", self.name).to_case(Case::Pascal)
    }

    fn fields(&self) -> Vec<StructField> {
        self.params
            .iter()
            .map(|field| {
                StructField::from(
                    field.column.as_ref().unwrap(),
                    field.number,
                    self.schemas,
                    self.default_schema,
                    &self.name(),
                    self.options,
                )
            })
            .collect::<Vec<_>>()
    }
}
