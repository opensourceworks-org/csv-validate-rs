use crossbeam_channel::unbounded;
use rayon::ThreadPoolBuilder;
use csv_validator_core::{IllegalCharacterValidator, Validator, reader::FastBufferedReader, ValidationIssue};
use std::sync::Arc;

fn main() -> std::io::Result<()> {
    let max_threads = 8;
    let batch_size = 100_000;
    let buffer_capacity = 8 * 1024 * 1024;

    let file_path = "examples/output.csv";
    let mut reader = FastBufferedReader::open(file_path, buffer_capacity)?;

    let (sender, receiver) = unbounded();

    let validators: Arc<Vec<Box<dyn Validator>>> = Arc::new(vec![
        Box::new(IllegalCharacterValidator::new("IllegalCharValidator", &["555555", "Zzzzz"]))
    ]);

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(max_threads)
        .build()
        .expect("ThreadPool setup failed");

    let mut batch = Vec::with_capacity(batch_size);
    let mut line_number = 0;

    while let Some(line) = reader.next_line()? {
        line_number += 1;
        batch.push((line_number, line.to_vec()));

        if batch.len() >= batch_size {
            process_batch(batch.drain(..).collect(), validators.clone(), &thread_pool, sender.clone());
        }
    }

    if !batch.is_empty() {
        process_batch(batch.drain(..).collect(), validators.clone(), &thread_pool, sender.clone());
    }

    drop(sender);

    // explicitly process the issues collected from batches
    for issue in receiver.iter().flatten() {
        println!(
            "[{}] Line {}, Position {:?}: {}",
            issue.validator, issue.line_number, issue.position, issue.message
        );
    }

    Ok(())
}

fn process_batch(
    lines: Vec<(usize, Vec<u8>)>,
    validators: Arc<Vec<Box<dyn Validator>>>,
    thread_pool: &rayon::ThreadPool,
    sender: crossbeam_channel::Sender<Vec<ValidationIssue>>,
) {
    thread_pool.spawn(move || {
        let mut issues_batch = Vec::with_capacity(lines.len() / 10);

        for (line_number, line) in lines {
            for validator in validators.iter() {
                validator.validate(&line, line_number, &mut issues_batch);
            }
        }

        sender.send(issues_batch).expect("send failed");
    });
}