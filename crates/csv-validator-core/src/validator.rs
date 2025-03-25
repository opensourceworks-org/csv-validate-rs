use crate::ValidationIssue;
use crossbeam_channel::Sender;


/// Validator explicitly takes &[u8] input and appends issues to provided issue vector.
pub trait Validator: Send + Sync {
    fn validate(&self, line: &[u8], line_number: usize, issues: &mut Vec<ValidationIssue>);
    fn name(&self) -> &'static str;
}