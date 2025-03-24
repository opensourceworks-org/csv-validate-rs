use crate::issue::{ValidationContext, ValidationResult};

pub trait Validator: Send + Sync {
    fn validate<'a>(
        &self, 
        input: &'a str, 
        context: &mut ValidationContext,
        line_number: usize, 
        fix: bool
    ) -> ValidationResult<'a>;
}