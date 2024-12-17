use options::Options;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use sqlc_sqlc_community_neoeinstein_prost::plugin;
use std::str::FromStr;
use syn::Ident;
use type_const::TypeConst;
use type_enum::TypeEnum;
use type_query::{QueryCommand, QueryValue, TypeQuery};
use type_struct::{StructTable, StructType, TypeStruct};

pub(crate) use multi_line::*;
pub(crate) use pg_data_type::*;

mod multi_line;
mod options;
mod pg_data_type;
mod type_const;
mod type_enum;
mod type_query;
mod type_struct;

pub(crate) fn list_tokenstream<'a>(list: &'a [String]) -> impl Iterator<Item = TokenStream> + 'a {
    list.iter().map(|item| {
        let mut tokens = TokenStream::new();
        for c in item.chars() {
            tokens.extend(crate::codegen::get_punct_from_char_tokens(c));
        }

        tokens
    })
}

pub fn get_ident(value: &str) -> Ident {
    format_ident!("{}", value)
}

fn build_query(
    query: &plugin::Query,
    schemas: &[plugin::Schema],
    default_schema: &str,
    structs: &mut Vec<TypeStruct>,
    options: &Options,
) -> TypeQuery {
    let query_cmd = QueryCommand::from_str(&query.cmd).expect("invalid query annotation");
    let is_batch = query_cmd.is_batch();

    // Query parameter limit, get it from the options
    let qpl = 3;
    let arg = QueryValue::from_query_params(
        &query.params,
        schemas,
        default_schema,
        structs,
        &query.name,
        qpl,
        is_batch,
        options,
    );

    let ret = QueryValue::from_query_columns(
        &query.columns,
        schemas,
        default_schema,
        structs,
        &query_cmd,
        &query.name,
        is_batch,
        options,
    );

    TypeQuery::new(&query.name, &query.cmd, arg, ret, options.use_async)
}

fn build_enums_from_schema<'a>(
    schema: &'a plugin::Schema,
    default_schema: &'a str,
    options: &'a Options,
) -> impl Iterator<Item = TypeEnum> + 'a {
    schema
        .enums
        .iter()
        .map(move |e| TypeEnum::from(e, &schema.name, default_schema, options))
}

fn build_structs_from_schema<'a>(
    schema: &'a plugin::Schema,
    default_schema: &'a str,
    options: &'a Options,
) -> impl Iterator<Item = TypeStruct> + 'a {
    schema.tables.iter().map(move |table| {
        StructType::Table(StructTable {
            table,
            schema,
            default_schema,
            options,
        })
        .into()
    })
}

#[derive(Default)]
pub struct CodePartials {
    enums: Vec<TypeEnum>,
    constants: Vec<TypeConst>,
    structs: Vec<TypeStruct>,
    queries: Vec<TypeQuery>,
}

impl CodePartials {
    fn sort_all(&mut self) {
        self.constants
            .sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.enums.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.queries.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
        self.structs.sort_by(|a, b| Ord::cmp(&a.name(), &b.name()));
    }
}

impl From<plugin::GenerateRequest> for CodePartials {
    fn from(req: plugin::GenerateRequest) -> Self {
        let options: Options = req.settings.expect("could not find sqlc config").into();
        let catalog = req.catalog.as_ref().unwrap();

        let mut code_partials = CodePartials::default();

        for schema in &catalog.schemas {
            if schema.name == "pg_catalog" || schema.name == "information_schema" {
                continue;
            }

            code_partials.enums.extend(build_enums_from_schema(
                schema,
                &catalog.default_schema,
                &options,
            ));

            code_partials.structs.extend(build_structs_from_schema(
                schema,
                &catalog.default_schema,
                &options,
            ));
        }

        for query in &req.queries {
            if query.name.is_empty() || query.cmd.is_empty() {
                continue;
            }

            code_partials.constants.push(query.into());

            let query = build_query(
                &query,
                &catalog.schemas,
                &catalog.default_schema,
                &mut code_partials.structs,
                &options,
            );
            code_partials.queries.push(query);
        }

        code_partials.sort_all();

        code_partials
    }
}

impl ToTokens for CodePartials {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let generated_comment = MultiLine(
            r#"
            /// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
            /// DO NOT EDIT.
"#,
        );

        let Self {
            enums,
            structs,
            constants,
            queries,
        } = self;

        tokens.extend(quote! {
            #generated_comment
            #(#constants)*
            #(#enums)*
            #(#structs)*
            #(#queries)*
        });
    }
}
