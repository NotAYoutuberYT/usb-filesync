use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    sync_directory: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sync_directory: Default::default(),
        }
    }
}
