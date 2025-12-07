use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use config::{Config as FileConfig, ConfigError, Environment, File};
use serde_derive::Deserialize;

use crate::scheduler::Job;

#[derive(Clone, Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub config: Option<PathBuf>,
    #[clap(short, long, default_value = "false")]
    pub version: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub log_path: Option<String>,
    pub tg_token: Option<String>,
    pub envs: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub jobs: HashMap<String, Job>,
}

impl Config {
    pub fn new(config: Option<PathBuf>) -> Result<Self, ConfigError> {
        let config = if let Some(path) = config {
            path
        } else {
            let home_dir = homedir::my_home()
                .map_err(|e| ConfigError::Message(format!("Failed to get home directory: {e}")))?;
            let home_dir = home_dir
                .ok_or_else(|| ConfigError::Message("Failed to get home directory".to_string()))?;
            let config_dir = home_dir.join(".config/crond");
            // create directory if it doesn't exist
            if !config_dir.exists() {
                std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
            }
            config_dir.join("config.toml")
        };
        let c = FileConfig::builder()
            .add_source(File::from(config))
            .add_source(Environment::with_prefix("CROND").separator("__"))
            .build()?;
        c.try_deserialize()
    }
}
