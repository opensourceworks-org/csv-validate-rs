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
                    "IllegalCharValidator",
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
                    "FieldCountValidator",
                    expected,
                    sep,
                )) as Box<dyn Validator>);
            }
            _ => {}
        }
    }

    out
}

pub fn build_validator_from_cli(args: &CliArgs) -> anyhow::Result<Vec<Box<dyn Validator>>> {
    let mut v: Vec<Box<dyn Validator>> = Vec::new();

    match args.validator.as_deref() {
        Some("illegal_chars") => {
            let chars: Vec<&str> = args
                .illegal_chars
                .as_ref()
                .map(|s| s.split(',').collect())
                .unwrap_or_else(|| vec!["@", "!"]);
            v.push(Box::new(IllegalCharactersValidator::new("IllegalCharValidator", &chars)));
        }
        Some("field_count") => {
            let expected = args.field_count.unwrap_or(10);
            v.push(Box::new(FieldCountValidator::new(
                "FieldCountValidator",
                expected,
                args.separator as u8,
            ))as Box<dyn Validator>);
        }
        Some("line_length") => {
            v.push(Box::new(LineLengthValidator::new(
                "LineLengthValidator",
                args.max_line_length,
            ))as Box<dyn Validator>);
        }
        _ => return Err(anyhow::anyhow!("Unknown or missing validator")),
    }

    Ok(v)
}
