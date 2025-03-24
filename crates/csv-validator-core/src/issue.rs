use std::borrow::Cow;

/// Single validation issue
#[derive(Debug)]
pub struct ValidationIssue {
    pub validator: &'static str,
    pub line_number: usize,
    pub position: Option<usize>,
    pub message: String,
}

/// Mutable context explicitly separate from line data
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
pub struct ValidationResult<'a> {
    pub line: Cow<'a, str>,
}

impl<'a> ValidationResult<'a> {
    pub fn new(line: &'a str) -> Self {
        Self { line: Cow::Borrowed(line) }
    }
}
