use serde::Deserialize;
use std::fs;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigLoaderError {
    #[error("Couldn't load config directory")]
    NoConfigDir(#[from] xdg::BaseDirectoriesError),
    #[error("Couldn't read config file")]
    ConfigFileRead(#[from] io::Error),
    #[error("Couldn't parse config YAML")]
    InvalidYaml(#[from] serde_yaml::Error),
}

#[derive(Deserialize, Debug)]
pub struct Config {
    inputs: Vec<ConfigInput>,
    outputs: Vec<ConfigOutput>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigInput {
    device: String,
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigOutput {
    axis_id: u16,
    combine_fn: String,
    inputs: Vec<ConfigInputAxis>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigInputAxis {
    js: String,
    axis: u16,
}

#[derive(Deserialize, Debug)]
pub enum ConfigCombineFn {
    Max,
    Hat {
        x: ConfigInputAxis,
        y: ConfigInputAxis,
    },
}

fn read_config_file() -> Result<String, ConfigLoaderError> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("composite_joystick")?;
    let config = fs::read_to_string(xdg_dirs.place_config_file("config.yaml")?)?;
    Ok(config)
}

pub fn load_config_file() -> Result<Config, ConfigLoaderError> {
    let config_string = read_config_file()?;
    let config = serde_yaml::from_str(&config_string)?;
    Ok(config)
}
