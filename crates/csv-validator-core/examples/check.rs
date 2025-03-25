use crossbeam_channel::unbounded;
use rayon::ThreadPoolBuilder;
use csv_validator_core::{
    IllegalCharactersValidator, FieldCountValidator, LineLengthValidator,
    Validator, ValidationIssue, OptimizedQuoteAwareReader, execute_validators
};
use std::sync::Arc;

fn main() -> std::io::Result<()> {
    let max_threads = 8;
    let batch_size = 100_000;
    let buffer_capacity = 8 * 1024 * 1024;

    let file_path = "examples/output.csv";
    let mut reader = OptimizedQuoteAwareReader::open(file_path, buffer_capacity)?;

    let (sender, receiver) = unbounded();

    let validators: Arc<Vec<Box<dyn Validator>>> = Arc::new(vec![
        Box::new(IllegalCharactersValidator::new("IllegalCharValidator", &[r#"137\n"#, "555555", "Zzzzz", "abcdef", "noway", "654321"])),
        Box::new(FieldCountValidator::new("FieldCountValidator", 50, b';')), // explicitly 10 fields expected
        Box::new(LineLengthValidator::new("LineLengthValidator", 1024)), // max line length explicitly 1024 bytes
    ]);

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(max_threads)
        .build()
        .expect("Failed to build thread pool");

    let mut batch = Vec::with_capacity(batch_size);
    let mut line_number = 0;
    let mut line_buf = Vec::with_capacity(1024);

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

    for issue_batch in receiver.iter() {
        for issue in issue_batch {
            println!(
                "[{}] Line {}, Position {:?}: {}",
                issue.validator, issue.line_number, issue.position, issue.message
            );
        }
    }

    Ok(())
}
