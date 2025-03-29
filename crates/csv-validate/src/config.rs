use serde::Deserialize;
use std::fs;
use csv_validator_core::{FieldCountValidator, IllegalCharactersValidator, LineLengthValidator, Validator};
use std::sync::Arc;
use crate::CliArgs;

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub common: Option<CommonConfig>,
    pub validators: Vec<ValidatorSpec>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommonConfig {
    pub quote_char: Option<char>,
    pub separator: Option<char>,
    pub has_header: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ValidatorSpec {
    #[serde(rename = "illegal_chars")]
    IllegalChars {
        illegal_chars: Vec<String>,
        replace_with: Vec<String>,
        fix: bool,
        enabled: bool,
        common: Option<CommonConfig>,
    },
    #[serde(rename = "field_count")]
    FieldCount {
        expected: usize,
        enabled: bool,
        common: Option<CommonConfig>,
    },
}



pub fn load_config(path: String) -> anyhow::Result<ConfigFile> {
    let content = fs::read_to_string(path)?;
    let config: ConfigFile = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn build_validators_from_config(config: ConfigFile) -> Vec<Box<dyn Validator>> {
    let mut out = Vec::new();

    for spec in config.validators {
        match spec {
            ValidatorSpec::IllegalChars {
                illegal_chars,
                enabled,
                ..
            } if enabled => {
                out.push(Box::new(IllegalCharactersValidator::new(
                    &illegal_chars.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
                ))as Box<dyn Validator>);
            }
            ValidatorSpec::FieldCount {
                expected,
                enabled,
                common,
            } if enabled => {
                let sep = common
                    .as_ref()
                    .and_then(|c| c.separator)
                    .unwrap_or(';') as u8;
                out.push(Box::new(FieldCountValidator::new(
                    expected,
                    sep,
                )) as Box<dyn Validator>);
            }
            _ => {}
        }
    }

    out
}

