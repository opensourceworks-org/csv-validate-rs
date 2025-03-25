use crate::{Validator, ValidationIssue};
use rayon::prelude::*;
use crossbeam_channel::Sender;
use std::sync::Arc;

pub fn execute_validators(
    lines: Vec<(usize, Vec<u8>)>,
    validators: Arc<Vec<Box<dyn Validator>>>,
    sender: Sender<Vec<ValidationIssue>>,
) {
    rayon::spawn(move || {
        let issues_batch: Vec<ValidationIssue> = lines.par_iter()
            .map(|(line_number, line)| {
                let mut local_issues = Vec::new();
                for validator in validators.iter() {
                    validator.validate(line, *line_number, &mut local_issues);
                }
                local_issues
            })
            .flatten()
            .collect();

        sender.send(issues_batch).expect("Issue sending failed");
    });
}