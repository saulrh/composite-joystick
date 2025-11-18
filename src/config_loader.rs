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

pub fn parse_config(config_string: &str) -> Result<Config, ConfigLoaderError> {
    let config = serde_yaml::from_str(config_string)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let yaml = r#"
inputs:
  - device: /dev/input/by-id/usb-joystick
    name: main_joystick
  - device: /dev/input/by-id/usb-throttle
    name: throttle

outputs:
  - axis_id: 0
    combine_fn: max
    inputs:
      - js: main_joystick
        axis: 0
      - js: throttle
        axis: 1
"#;

        let config = parse_config(yaml).expect("Failed to parse valid config");
        assert_eq!(config.inputs.len(), 2);
        assert_eq!(config.outputs.len(), 1);
        assert_eq!(config.inputs[0].device, "/dev/input/by-id/usb-joystick");
        assert_eq!(config.inputs[0].name, "main_joystick");
        assert_eq!(config.inputs[1].device, "/dev/input/by-id/usb-throttle");
        assert_eq!(config.inputs[1].name, "throttle");
        assert_eq!(config.outputs[0].axis_id, 0);
    }

    #[test]
    fn test_parse_empty_inputs() {
        let yaml = r#"
inputs: []
outputs: []
"#;

        let config = parse_config(yaml).expect("Failed to parse empty config");
        assert_eq!(config.inputs.len(), 0);
        assert_eq!(config.outputs.len(), 0);
    }

    #[test]
    fn test_parse_multiple_outputs() {
        let yaml = r#"
inputs:
  - device: /dev/input/js0
    name: js0

outputs:
  - axis_id: 0
    combine_fn: max
    inputs:
      - js: js0
        axis: 0
  - axis_id: 1
    combine_fn: max
    inputs:
      - js: js0
        axis: 1
  - axis_id: 2
    combine_fn: max
    inputs:
      - js: js0
        axis: 2
"#;

        let config = parse_config(yaml).expect("Failed to parse multiple outputs");
        assert_eq!(config.outputs.len(), 3);
        assert_eq!(config.outputs[0].axis_id, 0);
        assert_eq!(config.outputs[1].axis_id, 1);
        assert_eq!(config.outputs[2].axis_id, 2);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = r#"
inputs: [
  - device: /dev/input/js0
    name: js0
    missing_bracket
outputs: []
"#;

        let result = parse_config(yaml);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigLoaderError::InvalidYaml(_)));
    }

    #[test]
    fn test_parse_missing_required_field() {
        let yaml = r#"
inputs:
  - device: /dev/input/js0
    # missing 'name' field

outputs: []
"#;

        let result = parse_config(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_wrong_type() {
        let yaml = r#"
inputs:
  - device: 123
    name: js0

outputs: []
"#;

        let result = parse_config(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_config_with_multiple_input_axes() {
        let yaml = r#"
inputs:
  - device: /dev/input/js0
    name: js0
  - device: /dev/input/js1
    name: js1

outputs:
  - axis_id: 0
    combine_fn: max
    inputs:
      - js: js0
        axis: 0
      - js: js1
        axis: 0
"#;

        let config = parse_config(yaml).expect("Failed to parse config with multiple inputs");
        assert_eq!(config.inputs.len(), 2);
        assert_eq!(config.outputs[0].inputs.len(), 2);
        assert_eq!(config.outputs[0].inputs[0].js, "js0");
        assert_eq!(config.outputs[0].inputs[0].axis, 0);
        assert_eq!(config.outputs[0].inputs[1].js, "js1");
        assert_eq!(config.outputs[0].inputs[1].axis, 0);
    }

    #[test]
    fn test_config_loader_error_display() {
        // Test that error types can be created and displayed
        let yaml_error = serde_yaml::from_str::<Config>("invalid: [yaml").unwrap_err();
        let config_error = ConfigLoaderError::InvalidYaml(yaml_error);
        let error_string = format!("{}", config_error);
        assert!(error_string.contains("Couldn't parse config YAML"));
    }

    #[test]
    fn test_parse_special_characters_in_device_path() {
        let yaml = r#"
inputs:
  - device: "/dev/input/by-id/usb-Special_Device-1.0-event-joystick"
    name: "special-device"

outputs: []
"#;

        let config = parse_config(yaml).expect("Failed to parse special characters");
        assert_eq!(config.inputs[0].device, "/dev/input/by-id/usb-Special_Device-1.0-event-joystick");
        assert_eq!(config.inputs[0].name, "special-device");
    }
}
