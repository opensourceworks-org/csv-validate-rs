use crate::{ValidationIssue, Validator};
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
