use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref SETTINGS: Arc<RwLock<Settings>> =
        Arc::new(RwLock::new(Settings::new().expect("invalid configuration")));
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub host: String,
    pub port: usize,
    pub porep_partitions: u8,
    pub post_partitions: u8,
    pub metadata_dir: String,
    pub sealed_sector_dir: String,
    pub staged_sector_dir: String,
    pub max_num_staged_sectors: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            host: "127.0.0.1".into(),
            port: 9988,
            porep_partitions: 1,
            post_partitions: 2,
            metadata_dir: "meta".into(),
            sealed_sector_dir: "sealed".into(),
            staged_sector_dir: "staged".into(),
            max_num_staged_sectors: 10,
        }
    }
}

impl Settings {
    fn new() -> Result<Settings, config::ConfigError> {
        let mut s = config::Config::new();
        s.merge(config::File::with_name("filbase.config.toml").required(false))?;

        s.try_into()
    }

    pub fn load_config<S: AsRef<str>>(path: S) {
        let mut s = config::Config::new();
        s.merge(config::File::with_name(path.as_ref()).required(true))
            .unwrap();

        *SETTINGS.clone().write().unwrap() = s.try_into().unwrap();
    }

    pub fn server(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
