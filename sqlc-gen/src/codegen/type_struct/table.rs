use super::{Struct, StructField};
use crate::codegen::plugin;
use convert_case::{Case, Casing};

pub(crate) struct StructTable<'a> {
    pub table: &'a plugin::Table,
    pub default_schema: &'a str,
    pub schema: &'a plugin::Schema,
    pub options: &'a crate::codegen::Options,
}

impl<'a> Struct<'a> for StructTable<'a> {
    fn options(&self) -> &'a crate::codegen::Options {
        self.options
    }

    fn name(&self) -> String {
        let table_rel = self.table.rel.as_ref().unwrap();
        let mut table_name = table_rel.name.clone();
        if self.schema.name != self.default_schema {
            table_name = format!("{}_{table_name}", self.schema.name);
        }

        let pluralized_name = pluralizer::pluralize(table_name.as_str(), 1, false);
        pluralized_name.to_case(Case::Pascal)
    }

    fn fields(&self) -> Vec<StructField> {
        self.table
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                StructField::from(
                    col,
                    i as i32,
                    &[self.schema.clone()],
                    self.default_schema,
                    &self.name(),
                    self.options,
                )
            })
            .collect::<Vec<_>>()
    }

    fn table_identifier(&self) -> Option<plugin::Identifier> {
        let table_rel = self.table.rel.as_ref().unwrap();

        Some(plugin::Identifier {
            catalog: "".to_string(),
            schema: self.schema.name.clone(),
            name: table_rel.name.clone(),
        })
    }
}
