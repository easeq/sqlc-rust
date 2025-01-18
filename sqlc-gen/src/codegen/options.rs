use core::panic;
use serde::{Deserialize, Serialize};
use sqlc_sqlc_community_neoeinstein_prost::plugin;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Represents different types of children within a rule. The `ChildName` enum
/// is used to categorize child entities either as "all" or as specific named entities.
#[derive(Debug, Default, Deserialize, Serialize, Hash, Eq, PartialEq)]
enum ChildName {
    /// Rule applies to all children.
    #[serde(rename = "*")]
    #[default]
    All,

    /// Represents a specific child identified by a string name.
    #[serde(untagged)]
    Other(String),
}

/// A struct representing a child entity within a rule. A `Child` has a `name` (of type `ChildName`)
/// and a set of attributes.
/// Each `Child` represents either a field of a struct or a variant of an enum.
#[derive(Debug, Default, Deserialize, Serialize, Eq)]
struct Child {
    /// The name of the child (either "all" or a specific name).
    #[serde(rename = "name")]
    name: ChildName,

    /// A set of attributes associated with this child.
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

/// A struct that represents a rule that applies to a specific type (e.g., structs, enums).
/// A `Rule` contains the rule type (`typ`), optional `derive` and `container` attributes,
/// and optionally a set of `children` (fields or variants).
#[derive(Debug, Default, Deserialize, Serialize, Eq)]
pub(crate) struct Rule {
    /// The type of rule (e.g., structs, enums, or a wildcard rule for all types).
    #[serde(rename = "type")]
    typ: RuleType,

    /// An optional set of derives to be applied.
    derive: Option<HashSet<String>>,

    /// An optional set of container attributes.
    container: Option<HashSet<String>>,

    /// Optionally, the rule can apply to specific fields (children), such as struct fields or enum variants.
    #[serde(alias = "variants", alias = "fields")]
    children: Option<HashSet<Child>>,
}

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

/// Enum that represents different types of rules. These types can be applied to structs, enums,
/// or any other custom type. The `All` variant is used to apply a rule to all types.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub(crate) enum RuleType {
    /// A wildcard that applies to all rule types.
    #[serde(rename = "*")]
    #[default]
    All,

    /// A rule that specifically applies to structs.
    #[serde(rename = "structs")]
    Structs,

    /// A rule that specifically applies to enums.
    #[serde(rename = "enums")]
    Enums,

    /// A rule that applies to a custom type specified by a string.
    #[serde(untagged)]
    Other(String),
}

/// A struct that contains a set of rules. This struct is used to group together different rules
/// and provides methods to retrieve and apply the rules based on types or specific fields.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Rules(pub HashSet<Rule>);

impl Rules {
    /// Retrieves the "derive" attributes for a given rule type.
    ///
    /// # Parameters:
    /// - `typ`: The type of rule to retrieve derive attributes for (e.g., `RuleType::All`, `RuleType::Structs`).
    ///
    /// # Returns:
    /// A vector of derive strings associated with the rule type.
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

    /// Retrieves the "derive" attributes for a specific type and also the global "all" type.
    ///
    /// # Parameters:
    /// - `type_name`: The name of the type (e.g., a string representing a custom type).
    /// - `specific_type`: A specific type to retrieve derives for (e.g., `RuleType::Structs`).
    ///
    /// # Returns:
    /// A vector of derive strings for the specified type and global rules.
    pub(crate) fn derive_for<S: Into<String>>(
        &self,
        type_name: S,
        specific_type: RuleType,
    ) -> Vec<String> {
        let mut derives = self.get_derive_by_type(RuleType::All);
        derives.extend(self.get_derive_by_type(specific_type));
        derives.extend(self.get_derive_by_type(RuleType::Other(type_name.into())));
        derives
    }

    /// Retrieves the "container" attributes for a given rule type.
    ///
    /// # Parameters:
    /// - `typ`: The type of rule to retrieve container attributes for (e.g., `RuleType::All`, `RuleType::Structs`).
    ///
    /// # Returns:
    /// A vector of container attributes for the rule type.
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

    /// Retrieves the "container" attributes for a specific type and also the global "all" type.
    ///
    /// # Parameters:
    /// - `type_name`: The name of the type (e.g., a string representing a custom type).
    /// - `specific_type`: A specific type to retrieve container attributes for (e.g., `RuleType::Structs`).
    ///
    /// # Returns:
    /// A vector of container attributes for the specified type and global rules.
    pub(crate) fn container_attrs_for<S: Into<String>>(
        &self,
        type_name: S,
        specific_type: RuleType,
    ) -> Vec<String> {
        let mut attrs = self.get_container_attrs_by_type(RuleType::All);
        attrs.extend(self.get_container_attrs_by_type(specific_type));
        attrs.extend(self.get_container_attrs_by_type(RuleType::Other(type_name.into())));
        attrs
    }

    /// Retrieves the child attributes for a given child name and rule type.
    ///
    /// # Parameters:
    /// - `name`: The name of the child to retrieve attributes for (e.g., `ChildName::All`, `ChildName::Other`).
    /// - `typ`: The type of rule to retrieve child attributes for.
    ///
    /// # Returns:
    /// A vector of child attributes for the specified child name and rule type.
    fn get_child_attrs_by_type(&self, name: ChildName, typ: RuleType) -> Vec<String> {
        let mut result = vec![];
        if let Some(rule) = self.0.get(&typ) {
            result.extend(if let Some(ref children) = &rule.children {
                children
                    .iter()
                    .filter_map(|child| {
                        if child.name != name {
                            None
                        } else {
                            Some(child.attributes.clone())
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            } else {
                vec![]
            });
        }
        result
    }

    /// Retrieves the child attributes for a specific child and rule type.
    ///
    /// # Parameters:
    /// - `child_name`: The name of the specific child (e.g., a string).
    /// - `type_name`: The name of the type (e.g., a string representing a custom type).
    /// - `specific_type`: The specific type of rule to apply.
    ///
    /// # Returns:
    /// A vector of child attributes for the specified child and rule types.
    pub(crate) fn child_attr_for(
        &self,
        child_name: String,
        type_name: String,
        specific_type: RuleType,
    ) -> Vec<String> {
        let mut attrs = Vec::new();

        // Get rules that apply to all struct fields and enum variants
        attrs.extend(self.get_child_attrs_by_type(ChildName::All, RuleType::All));
        // Add rules specific to either all fields of structs or all variants of enums
        attrs.extend(self.get_child_attrs_by_type(ChildName::All, specific_type.clone()));
        // Add rules for specific fields/variants of either all structs or all enums
        attrs.extend(
            self.get_child_attrs_by_type(ChildName::Other(child_name.clone()), specific_type),
        );
        // Add rules all fields/variants of the given type
        attrs.extend(
            self.get_child_attrs_by_type(ChildName::All, RuleType::Other(type_name.clone())),
        );
        // Add rules specific fields/variants of the given type
        attrs.extend(
            self.get_child_attrs_by_type(ChildName::Other(child_name), RuleType::Other(type_name)),
        );

        attrs
    }
}

/// Options structure that holds settings for code generation, like whether to use async
/// or deadpool, and the associated rules for codegen.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Options {
    /// Flag indicating whether to use async in code generation.
    #[serde(default)]
    pub use_async: bool,

    /// Flag indicating whether to use deadpool in code generation.
    #[serde(default)]
    pub use_deadpool: bool,

    /// Optional set of rules to apply for code generation.
    #[serde(default)]
    pub rules: Option<Rules>,
}

impl From<&plugin::Settings> for Options {
    /// Converts settings from the plugin into `Options`. If the settings or `codegen` options are missing or invalid,
    /// the function will panic with a descriptive error message.
    ///
    /// # Parameters:
    /// - `settings`: The plugin settings to extract codegen options from.
    ///
    /// # Returns:
    /// An `Options` struct that holds the codegen settings.
    fn from(settings: &plugin::Settings) -> Self {
        // Retrieve codegen settings or panic if not found
        let codegen = settings
            .codegen
            .as_ref()
            .expect("codegen settings not defined in sqlc config");

        // Attempt to convert the byte array to a UTF-8 string
        let options_str = match std::str::from_utf8(&codegen.options) {
            Ok(v) => v,
            Err(e) => {
                // Return an error instead of panicking
                panic!("Invalid UTF-8 sequence in codegen options: {}", e)
            }
        };

        // Deserialize the options string if it's not empty, otherwise return default
        if options_str.is_empty() {
            Options::default()
        } else {
            // Try to deserialize the options, and handle errors gracefully
            serde_json::from_str(options_str).unwrap_or_else(|e| {
                panic!(
                    "Failed to deserialize codegen options from JSON. Error: {}\n\
                    Expected a valid object. Default options: {}",
                    e,
                    serde_json::to_string(&Options::default()).unwrap_or_else(|_| "{}".to_string())
                )
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a sample Rule with derives and container attributes.
    fn create_sample_rule(typ: RuleType) -> Rule {
        Rule {
            typ,
            derive: Some(
                vec!["Deserialize".to_string(), "Serialize".to_string()]
                    .into_iter()
                    .collect(),
            ),
            container: Some(
                vec!["Container1".to_string(), "Container2".to_string()]
                    .into_iter()
                    .collect(),
            ),
            children: Some(
                vec![
                    Child {
                        name: ChildName::Other("child1".to_string()),
                        attributes: vec!["attr1".to_string(), "attr2".to_string()]
                            .into_iter()
                            .collect(),
                    },
                    Child {
                        name: ChildName::Other("child2".to_string()),
                        attributes: vec!["attr3".to_string()].into_iter().collect(),
                    },
                ]
                .into_iter()
                .collect(),
            ),
        }
    }

    #[test]
    fn test_get_derive_by_type() {
        let mut rules = Rules(HashSet::new());

        // Add a sample rule
        let rule = create_sample_rule(RuleType::Structs);
        rules.0.insert(rule);

        // Test retrieving derives for RuleType::All
        let derives_all = rules.get_derive_by_type(RuleType::All);
        assert_eq!(derives_all, vec!["Deserialize", "Serialize"]);

        // Test retrieving derives for RuleType::Structs
        let derives_structs = rules.get_derive_by_type(RuleType::Structs);
        assert_eq!(derives_structs, vec!["Deserialize", "Serialize"]);
    }

    #[test]
    fn test_derive_for() {
        let mut rules = Rules(HashSet::new());

        // Add a sample rule
        let rule = create_sample_rule(RuleType::Structs);
        rules.0.insert(rule);

        // Test derive_for with a specific type and type name
        let derives = rules.derive_for("MyStruct", RuleType::Structs);
        assert_eq!(derives, vec!["Deserialize", "Serialize"]);

        // Test derive_for with RuleType::All
        let derives_all = rules.derive_for("MyEnum", RuleType::All);
        assert_eq!(derives_all, vec!["Deserialize", "Serialize"]);
    }

    #[test]
    fn test_get_container_attrs_by_type() {
        let mut rules = Rules(HashSet::new());

        // Add a sample rule
        let rule = create_sample_rule(RuleType::Structs);
        rules.0.insert(rule);

        // Test retrieving container attributes for RuleType::All
        let container_all = rules.get_container_attrs_by_type(RuleType::All);
        assert_eq!(container_all, vec!["Container1", "Container2"]);

        // Test retrieving container attributes for RuleType::Structs
        let container_structs = rules.get_container_attrs_by_type(RuleType::Structs);
        assert_eq!(container_structs, vec!["Container1", "Container2"]);
    }

    #[test]
    fn test_container_attrs_for() {
        let mut rules = Rules(HashSet::new());

        // Add a sample rule
        let rule = create_sample_rule(RuleType::Structs);
        rules.0.insert(rule);

        // Test container_attrs_for with a specific type
        let container_attrs = rules.container_attrs_for("MyStruct", RuleType::Structs);
        assert_eq!(container_attrs, vec!["Container1", "Container2"]);

        // Test container_attrs_for with RuleType::All
        let container_all = rules.container_attrs_for("MyEnum", RuleType::All);
        assert_eq!(container_all, vec!["Container1", "Container2"]);
    }

    #[test]
    fn test_get_child_attrs_by_type() {
        let mut rules = Rules(HashSet::new());

        // Add a sample rule
        let rule = create_sample_rule(RuleType::Structs);
        rules.0.insert(rule);

        // Test retrieving child attributes for RuleType::All and ChildName::All
        let child_attrs_all = rules.get_child_attrs_by_type(ChildName::All, RuleType::All);
        assert_eq!(child_attrs_all, vec!["attr1", "attr2", "attr3"]);

        // Test retrieving child attributes for a specific child (e.g., "child1")
        let child_attrs_child1 =
            rules.get_child_attrs_by_type(ChildName::Other("child1".to_string()), RuleType::All);
        assert_eq!(child_attrs_child1, vec!["attr1", "attr2"]);
    }

    #[test]
    fn test_child_attr_for() {
        let mut rules = Rules(HashSet::new());

        // Add a sample rule
        let rule = create_sample_rule(RuleType::Structs);
        rules.0.insert(rule);

        // Test child_attr_for with a specific child and type name
        let child_attrs = rules.child_attr_for(
            "child1".to_string(),
            "MyStruct".to_string(),
            RuleType::Structs,
        );
        assert_eq!(child_attrs, vec!["attr1", "attr2"]);

        // Test child_attr_for with RuleType::All
        let child_attrs_all =
            rules.child_attr_for("child1".to_string(), "MyStruct".to_string(), RuleType::All);
        assert_eq!(child_attrs_all, vec!["attr1", "attr2"]);

        // Test child_attr_for with RuleType::Other (custom type)
        let child_attrs_custom = rules.child_attr_for(
            "child2".to_string(),
            "MyCustomType".to_string(),
            RuleType::Other("CustomType".to_string()),
        );
        assert_eq!(child_attrs_custom, vec!["attr3"]);
    }

    #[test]
    fn test_options_from_plugin_settings() {
        // Simulate a plugin settings conversion
        let plugin_settings = plugin::Settings {
            codegen: Some(plugin::Codegen {
                options: b"{\"use_async\":true,\"use_deadpool\":false}".to_vec(),
                ..plugin::Codegen::default()
            }),
            ..Default::default()
        };

        let options: Options = Options::from(&plugin_settings);

        // Verify that the options were correctly parsed
        assert_eq!(options.use_async, true);
        assert_eq!(options.use_deadpool, false);
    }

    #[test]
    fn test_invalid_options_json() {
        // Simulate invalid JSON in the plugin settings
        let plugin_settings = plugin::Settings {
            codegen: Some(plugin::Codegen {
                options: b"{use_async:true}".to_vec(), // Invalid JSON format (missing quotes)
                ..plugin::Codegen::default()
            }),
            ..Default::default()
        };

        // Check that it panics with the expected error
        let result = std::panic::catch_unwind(|| {
            let _: Options = Options::from(&plugin_settings);
        });

        assert!(result.is_err());
    }
}
