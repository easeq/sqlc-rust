use core::panic;
use serde::{Deserialize, Serialize};
use sqlc_sqlc_community_neoeinstein_prost::plugin;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Options {
    #[serde(default)]
    pub use_async: bool,

    #[serde(default)]
    use_deadpool: bool,
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
        }

        options
    }
}
