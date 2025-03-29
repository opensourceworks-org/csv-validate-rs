use std::fs::File;
use crate::{Validator, ValidationIssue, OptimizedQuoteAwareReader};
use rayon::prelude::*;
use crossbeam_channel::{unbounded, Sender};
use std::sync::Arc;
use rayon::ThreadPoolBuilder;

pub fn validate_file_with_config(
    filename: &str,
    validators: Arc<Vec<Box<dyn Validator>>>,
    threads: usize,
    batch_size: usize,
    buffer_size: usize,
) -> Result<Vec<ValidationIssue>, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let mut reader = OptimizedQuoteAwareReader::new(file, buffer_size);
    let pool = ThreadPoolBuilder::new().num_threads(threads).build()?;
    let (sender, receiver) = unbounded();

    let mut line_buf = Vec::with_capacity(1024);
    let mut batch = Vec::with_capacity(batch_size);
    let mut line_number = 0;

    while let Some(line) = reader.next_logical_line(&mut line_buf)? {
        line_number += 1;
        batch.push((line_number, line.to_vec()));

        if batch.len() >= batch_size {
            execute_validators(batch.drain(..).collect(), validators.clone(), sender.clone());
        }
    }

    if !batch.is_empty() {
        execute_validators(batch.drain(..).collect(), validators.clone(), sender.clone());
    }

    drop(sender);

    let mut issues = Vec::new();
    for chunk in receiver.iter() {
        issues.extend(chunk);
    }

    Ok(issues)
}


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