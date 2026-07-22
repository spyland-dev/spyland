use serde::{Deserialize, Serialize};
use spyland_lib::config::ConfigSection;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub sort_ascending: bool,
    pub sort_by_time: bool,
}

impl ConfigSection for Config {
    const SECTION: &'static str = "frontend.cli";
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sort_ascending: true,
            sort_by_time: true,
        }
    }
}
