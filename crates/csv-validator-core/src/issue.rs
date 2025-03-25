use std::borrow::Cow;

#[derive(Debug)]
pub struct ValidationIssue {
    pub validator: &'static str,
    pub line_number: usize,
    pub position: Option<usize>,
    pub message: String,
}

pub struct ValidationContext {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Validation result explicitly carries forward the current line
/// no longer needed since only check-only mode
pub struct ValidationResult<'a> {
    pub line: Cow<'a, str>,
}

impl<'a> ValidationResult<'a> {
    pub fn new(line: &'a str) -> Self {
        Self {
            line: Cow::Borrowed(line),
        }
    }
}
