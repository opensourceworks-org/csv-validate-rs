use aho_corasick::AhoCorasick;
use crate::{ValidationIssue, Validator};
use crossbeam_channel::Sender;

pub struct IllegalCharacterValidator {
    name: &'static str,
    matcher: AhoCorasick,
}

impl IllegalCharacterValidator {
    pub fn new(name: &'static str, illegal_chars: &[&str]) -> Self {
        Self {
            name,
            matcher: AhoCorasick::new(illegal_chars).unwrap(),
        }
    }
}

impl Validator for IllegalCharacterValidator {
    fn validate(
        &self,
        line: &str,
        line_number: usize,
        sender: &Sender<ValidationIssue>,
    ) {
        for mat in self.matcher.find_iter(line) {
            sender.send(ValidationIssue {
                validator: self.name,
                line_number,
                position: Some(mat.start()),
                message: format!("Illegal character found: '{}'", &line[mat.start()..mat.end()]),
            }).expect("Send issue failed");
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
}
