use aho_corasick::AhoCorasick;
use crate::{issue::ValidationContext, ValidationIssue, ValidationResult, Validator};
use std::borrow::Cow;

/// Validator that checks for illegal characters, optionally replacing them.
pub struct IllegalCharacterValidator {
    matcher: AhoCorasick,
    replacements: Vec<String>,
}

impl IllegalCharacterValidator {
    pub fn new(illegal_chars: &[&str], replacements: &[&str]) -> Self {
        assert_eq!(
            illegal_chars.len(),
            replacements.len(),
            "Each illegal char must have a replacement"
        );
        Self {
            matcher: AhoCorasick::new(illegal_chars).unwrap(),
            replacements: replacements.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Validator for IllegalCharacterValidator {
    fn validate<'a>(
        &self, 
        input: &'a str,
        context: &mut ValidationContext,
        line_number: usize, 
        fix: bool
    ) -> ValidationResult<'a> {
        let matches: Vec<_> = self.matcher.find_iter(input).collect();

        if matches.is_empty() {
            return ValidationResult {
                line: Cow::Borrowed(input),
                modified: false,
            };
        }

        let mut line_cow: Cow<str> = Cow::Borrowed(input);
        let mut modified = false;

        for m in matches {
            let illegal_str = &input[m.start()..m.end()];
            let replacement = &self.replacements[m.pattern()];

            context.add_issue(ValidationIssue {
                line_number,
                position: Some(m.start()),
                message: format!("Illegal char found: '{}'", illegal_str),
                fixed: fix,
            });

            if fix {
                if let Cow::Borrowed(_) = line_cow {
                    line_cow = Cow::Owned(input.to_owned());
                }

                if let Cow::Owned(ref mut owned_line) = line_cow {
                    *owned_line = owned_line.replace(illegal_str, replacement);
                    modified = true;
                }
            }
        }

        ValidationResult {
            line: line_cow,
            modified,
        }
    }
}