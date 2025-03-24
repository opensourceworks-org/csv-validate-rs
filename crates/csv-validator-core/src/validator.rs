use crate::ValidationIssue;
use crossbeam_channel::Sender;

/// Trait for validators in check-only mode, clearly reporting issues
pub trait Validator: Send + Sync {
    fn validate(
        &self,
        line: &str,
        line_number: usize,
        sender: &Sender<ValidationIssue>,
    );

    fn name(&self) -> &'static str;
}
