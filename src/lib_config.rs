//! Deals with configurations.
//!
//! All functions in this module panic.

use std::fs;
use std::io::Read;

use serde::Deserialize;
use toml;
use once_cell::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::new();


#[derive(Deserialize)]
#[derive(Debug)]
pub struct Config {
    pub app: AppConfig,
    pub restic: ResticConfig,
    pub sources: Vec<SourceConfig>,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct AppConfig {
    pub log_dir: String,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct ResticConfig {
    pub exe_path: String,
    pub repo_dir: String,
    pub cache_dir: String,
    pub password_file: String,
}


#[derive(Deserialize)]
#[derive(Debug)]
pub struct SourceConfig {
    pub vg: String,
    pub lv: String,
    pub snapshot_size: String,
}

pub fn setup(p: &str) {
    let config = load(p);
    CONFIG.set(config).expect("Setting up config");
}


pub fn get() -> &'static Config {
    CONFIG.get().expect("Config is not initialized")
}


fn load(p: &str) -> Config {
    let mut f = fs::File::open(p).expect("Opening config file");
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).expect("Reading config file");

    let config = toml::from_slice(&bytes).expect("Parsing config");

    config
}