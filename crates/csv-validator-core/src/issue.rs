use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    pub line_number: usize,
    pub position: Option<usize>,
    pub message: String,
    pub fixed: bool,
}

/// Mutable context explicitly separated from line data
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

/// Represents validator results explicitly, minimal allocation
pub struct ValidationResult<'a> {
    pub line: Cow<'a, str>,
    pub modified: bool,
}
