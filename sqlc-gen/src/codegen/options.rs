use crate::codegen::{TypeEnum, TypeStruct};
use core::panic;
use serde::{Deserialize, Serialize};
use sqlc_sqlc_community_neoeinstein_prost::plugin;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Default, Deserialize, Serialize, Hash, Eq, PartialEq)]
enum ChildName {
    #[serde(rename = "*")]
    #[default]
    All,

    // #[serde(untagged)]
    // Exclude(String),
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Default, Deserialize, Serialize, Eq)]
struct Child {
    #[serde(rename = "name")]
    name: ChildName,
    attributes: HashSet<String>,
}

impl Hash for Child {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Child {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Borrow<ChildName> for Child {
    fn borrow(&self) -> &ChildName {
        &self.name
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Eq)]
struct Rule {
    #[serde(rename = "type")]
    typ: RuleType,
    derive: Option<HashSet<String>>,
    container: Option<HashSet<String>>,
    #[serde(alias = "variants", alias = "fields")]
    children: Option<HashSet<Child>>,
}

// impl Rule {
//     fn empty_from_type(typ: RuleType) -> Self {
//         Self {
//             typ,
//             ..Self::default()
//         }
//     }
// }

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.typ.hash(state);
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.typ == other.typ
    }
}

impl Borrow<RuleType> for Rule {
    fn borrow(&self) -> &RuleType {
        &self.typ
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub(crate) enum RuleType {
    #[serde(rename = "*")]
    #[default]
    All,
    #[serde(rename = "structs")]
    Structs,
    #[serde(rename = "enums")]
    Enums,

    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Rules(HashSet<Rule>);

impl Rules {
    fn get_derive_by_type(&self, typ: RuleType) -> Vec<String> {
        let mut result = vec![];
        if let Some(rule) = self.0.get(&typ) {
            result.extend(if let Some(derive) = &rule.derive {
                derive.clone().into_iter().collect::<Vec<_>>()
            } else {
                vec![]
            });
        }

        result
    }

    pub(crate) fn derive_for(&self, type_name: String, specific_type: RuleType) -> Vec<String> {
        let mut derives = self.get_derive_by_type(RuleType::All);
        derives.extend(self.get_derive_by_type(specific_type));
        derives.extend(self.get_derive_by_type(RuleType::Other(type_name)));

        derives
    }

    fn get_container_attrs_by_type(&self, typ: RuleType) -> Vec<String> {
        let mut result = vec![];
        if let Some(rule) = self.0.get(&typ) {
            result.extend(if let Some(container) = &rule.container {
                container.clone().into_iter().collect::<Vec<_>>()
            } else {
                vec![]
            });
        }

        result
    }

    pub(crate) fn container_attrs_for(
        &self,
        type_name: String,
        specific_type: RuleType,
    ) -> Vec<String> {
        let mut attrs = self.get_container_attrs_by_type(RuleType::All);
        attrs.extend(self.get_container_attrs_by_type(specific_type));
        attrs.extend(self.get_container_attrs_by_type(RuleType::Other(type_name)));

        attrs
    }

    // fn get_child_attrs_by_type(&self, name: ChildName, type_name: String, typ: RuleType) -> Vec<String> {
    //     let mut result = vec![];
    //     if let Some(rule) = self.0.get(&typ) {
    //         result.extend(if let Some(container) = &rule.container {
    //             container.clone().into_iter().collect::<Vec<_>>()
    //         } else {
    //             vec![]
    //         });
    //     }
    //
    //     result
    // }
    //
    // pub(crate) child_attr_for(&self, child_name: String, specific_type: Child) -> Vec<String> {
    //     let mut attrs = self.get_container_attrs_by_type(ChildName::All);
    // }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Options {
    #[serde(default)]
    pub use_async: bool,

    #[serde(default)]
    pub use_deadpool: bool,

    #[serde(default)]
    pub rules: Option<Rules>,
}

impl From<plugin::Settings> for Options {
    fn from(settings: plugin::Settings) -> Self {
        let codegen = settings
            .codegen
            .as_ref()
            .expect("codegen settings not defined in sqlc config");
        let options_str = match std::str::from_utf8(&codegen.options) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence in codegen options: {}", e),
        };

        let mut options = Options::default();
        if !options_str.is_empty() {
            options = serde_json::from_str(options_str).expect(
                format!(
                    "could not deserialize codegen options (valid object: {:?})",
                    serde_json::to_string(&Options::default())
                        .expect("could not convert options to json string"),
                )
                .as_str(),
            );
            // panic!("{:#?}", options);
        }

        options
    }
}
