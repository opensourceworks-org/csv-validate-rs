use crate::{FieldCountValidator, Validator, IllegalCharactersValidator, LineLengthValidator};

#[derive(Debug, Clone)]
pub enum ValidatorSpec {
    IllegalChars {
        illegal_chars: Vec<String>,
        enabled: bool,
    },
    FieldCount {
        expected: usize,
        enabled: bool,
    },
    LineLength {
        enabled: bool,
        max_length: usize,
    }
}

impl ValidatorSpec {
    pub fn new_illegal_chars(chars: Vec<String>) -> Self {
        Self::IllegalChars {
            illegal_chars: chars,
            enabled: true,
        }
    }

    pub fn new_field_count(expected: usize) -> Self {
        Self::FieldCount {
            expected,
            enabled: true,
        }
    }

    pub fn new_line_length(max_length: usize) -> Self {
        Self::LineLength {
            max_length,
            enabled: true,
        }
    }

    pub fn into_validator(self, separator: u8) -> Box<dyn Validator> {
        match self {
            ValidatorSpec::IllegalChars { illegal_chars, .. } => {
                let refs = illegal_chars.iter().map(AsRef::as_ref).collect::<Vec<_>>();
                Box::new(IllegalCharactersValidator::new( &refs))
            }
            ValidatorSpec::FieldCount { expected, .. } => {
                Box::new(FieldCountValidator::new( expected, separator))
            }
            ValidatorSpec::LineLength { max_length, .. } => {
                Box::new(LineLengthValidator::new( max_length))
            }
        }
    }
}
