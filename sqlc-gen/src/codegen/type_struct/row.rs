use super::{Struct, StructField};
use crate::codegen::plugin;
use convert_case::{Case, Casing};

pub(crate) struct StructRow<'a> {
    pub name: &'a str,
    pub columns: &'a [plugin::Column],
    pub default_schema: &'a str,
    pub schemas: &'a [plugin::Schema],
    pub options: &'a crate::codegen::Options,
}

impl<'a> Struct<'a> for StructRow<'a> {
    fn options(&self) -> &'a crate::codegen::Options {
        self.options
    }

    fn name(&self) -> String {
        format!("{}Row", self.name).to_case(Case::Pascal)
    }

    fn fields(&self) -> Vec<StructField> {
        self.columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                StructField::from(
                    col,
                    i as i32,
                    self.schemas,
                    self.default_schema,
                    &self.name(),
                    self.options,
                )
            })
            .collect::<Vec<_>>()
    }
}
