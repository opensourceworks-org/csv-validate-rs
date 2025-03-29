use crate::{OptimizedQuoteAwareReader, Validator, ValidationIssue, execute_validators};
use std::{fs::File, sync::Arc, io::Result};
use rayon::ThreadPoolBuilder;
use crossbeam_channel::unbounded;

pub struct ValidationOptions {
    pub threads: usize,
    pub batch_size: usize,
    pub buffer_size: usize,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            threads: 8,
            batch_size: 100_000,
            buffer_size: 8 * 1024 * 1024,
        }
    }
}

pub fn validate_file(
    path: &str,
    validators: Arc<Vec<Box<dyn Validator>>>,
    options: ValidationOptions,
) -> Result<Vec<ValidationIssue>> {
    let file = File::open(path)?;
    let mut reader = OptimizedQuoteAwareReader::new(file, options.buffer_size);
    let thread_pool = ThreadPoolBuilder::new().num_threads(options.threads).build().unwrap();
    let (sender, receiver) = unbounded();

    let mut line_buf = Vec::with_capacity(1024);
    let mut batch = Vec::with_capacity(options.batch_size);
    let mut line_number = 0;

    while let Some(line) = reader.next_logical_line(&mut line_buf)? {
        line_number += 1;
        batch.push((line_number, line.to_vec()));

        if batch.len() >= options.batch_size {
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
