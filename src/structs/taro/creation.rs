use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaroCreationConfig {
    path: String,
}

impl Default for TaroCreationConfig {
    fn default() -> Self {
        Self {
            path: "default".to_owned(),
        }
    }
}
