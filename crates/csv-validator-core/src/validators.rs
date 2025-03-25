pub use crate::{ValidationIssue, Validator};
use aho_corasick::AhoCorasick;

pub struct IllegalCharactersValidator {
    name: &'static str,
    matcher: AhoCorasick,
}

impl IllegalCharactersValidator {
    pub fn new(name: &'static str, illegal_chars: &[&str]) -> Self {
        Self {
            name,
            matcher: AhoCorasick::new(illegal_chars).unwrap(),
        }
    }
}

impl Validator for IllegalCharactersValidator {

    fn validate(&self, line: &[u8], line_number: usize, issues: &mut Vec<ValidationIssue>) {
        for mat in self.matcher.find_iter(line) {
            let illegal_str = &line[mat.start()..mat.end()];
            let illegal_char = std::str::from_utf8(illegal_str).unwrap_or("<invalid utf8>");

            issues.push(ValidationIssue {
                validator: self.name,
                line_number,
                position: Some(mat.start()),
                message: format!("Illegal character(s) '{}'", illegal_char),
            });
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

pub struct FieldCountValidator {
    name: &'static str,
    expected_fields: usize,
    delimiter: u8,
}

impl FieldCountValidator {
    pub fn new(name: &'static str, expected_fields: usize, delimiter: u8) -> Self {
        Self { name, expected_fields, delimiter }
    }
}

impl Validator for FieldCountValidator {
    fn validate(&self, line: &[u8], line_number: usize, issues: &mut Vec<ValidationIssue>) {
        let actual_fields = bytecount::count(line, self.delimiter) + 1;
        if actual_fields != self.expected_fields {
            issues.push(ValidationIssue {
                validator: self.name,
                line_number,
                position: None,
                message: format!("Expected {} fields, found {}", self.expected_fields, actual_fields),
            });
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

pub struct LineLengthValidator {
    name: &'static str,
    max_length: usize,
}

impl LineLengthValidator {
    pub fn new(name: &'static str, max_length: usize) -> Self {
        Self { name, max_length }
    }
}

impl Validator for LineLengthValidator {
    fn validate(&self, line: &[u8], line_number: usize, issues: &mut Vec<ValidationIssue>) {
        if line.len() > self.max_length {
            issues.push(ValidationIssue {
                validator: self.name,
                line_number,
                position: None,
                message: format!("Line length {} exceeds maximum {}", line.len(), self.max_length),
            });
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
}
