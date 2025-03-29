pub use crate::{ValidationIssue, Validator};
use aho_corasick::AhoCorasick;

#[derive(Clone)]
pub struct IllegalCharactersValidator {
    matcher: AhoCorasick,
}

impl IllegalCharactersValidator {
    pub fn new<'a>(patterns: &[&'a str]) -> Self {
        let matcher = AhoCorasick::new(patterns).expect("failed to build Aho-Corasick matcher");
        Self { matcher }
    }

    fn clone_box(&self) -> Box<dyn Validator> {
        println!("Cloning validator: {}", self.name());

        Box::new((*self).clone())
    }
}

impl Validator for IllegalCharactersValidator {

    fn validate(&self, line: &[u8], line_number: usize, issues: &mut Vec<ValidationIssue>) {
        for mat in self.matcher.find_iter(line) {
            let illegal_str = &line[mat.start()..mat.end()];
            let illegal_char = std::str::from_utf8(illegal_str).unwrap_or("<invalid utf8>");

            issues.push(ValidationIssue {
                validator: self.name(),
                line_number,
                position: Some(mat.start()),
                message: format!("Illegal character(s) '{}'", illegal_char),
            });
        }
    }

    fn name(&self) -> &'static str {
        "illegal_characters"
    }

    fn clone_box(&self) -> Box<dyn Validator> {
        println!("Cloning validator: {}", self.name());

        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct FieldCountValidator {
    expected_fields: usize,
    delimiter: u8,
}

impl FieldCountValidator {
    pub fn new(expected_fields: usize, delimiter: u8) -> Self {
        Self {expected_fields, delimiter }
    }
}

impl Validator for FieldCountValidator {
    fn validate(&self, line: &[u8], line_number: usize, issues: &mut Vec<ValidationIssue>) {
        let actual_fields = bytecount::count(line, self.delimiter) + 1;
        if actual_fields != self.expected_fields {
            issues.push(ValidationIssue {
                validator: self.name(),
                line_number,
                position: None,
                message: format!("Expected {} fields, found {}", self.expected_fields, actual_fields),
            });
        }
    }

    fn name(&self) -> &'static str {
       "field_count"
    }

    fn clone_box(&self) -> Box<dyn Validator> {
        println!("Cloning validator: {}", self.name());

        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct LineLengthValidator {
    max_length: usize,
}

impl LineLengthValidator {
    pub fn new( max_length: usize) -> Self {
        Self {  max_length }
    }
}

impl Validator for LineLengthValidator {
    fn validate(&self, line: &[u8], line_number: usize, issues: &mut Vec<ValidationIssue>) {
        if line.len() > self.max_length {
            issues.push(ValidationIssue {
                validator: self.name(),
                line_number,
                position: None,
                message: format!("Line length {} exceeds maximum {}", line.len(), self.max_length),
            });
        }
    }

    fn name(&self) -> &'static str {
       "line_length"
    }

    fn clone_box(&self) -> Box<dyn Validator> {
        println!("Cloning validator: {}", self.name());

        Box::new(self.clone())
    }
}
