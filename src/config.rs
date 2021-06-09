extern crate toml;

use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub location: Vec<Location>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Location {
    pub name: String,
    pub tz: String,
}

pub fn read_config(config_path: &str) -> Result<Config, Box<dyn Error>> {
    let path = Path::new(config_path);
    let mut file = File::open(path)?;
    let mut config = String::new();
    file.read_to_string(&mut config)?;
    Ok(toml::from_str(config.as_str())?)
}
