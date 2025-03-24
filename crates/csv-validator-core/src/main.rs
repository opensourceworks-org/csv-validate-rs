use crossbeam_channel::{unbounded, Sender};
use rayon::ThreadPoolBuilder;
use csv_validator_core::{IllegalCharacterValidator, Validator};
use std::sync::Arc;

fn main() {
    let lines = vec![
        "hello world",
        "hello@world",
        "good!morning",
        "clean_line",
        "another@line!",
        "more clean data",
    ];

    // Explicitly define resource management limits
    let max_threads = 4; // Explicit thread control
    let batch_size = 2;  // Explicit batching

    // Explicitly setup channel for reporting issues
    let (sender, receiver) = unbounded();

    // Validators
    let validators: Arc<Vec<Box<dyn Validator>>> = Arc::new(vec![
        Box::new(IllegalCharacterValidator::new("IllegalCharValidator", &["@", "!"]))
    ]);

    // Configure rayon thread pool explicitly
    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(max_threads)
        .build()
        .expect("ThreadPool failed");

    // Process explicitly in batches
    for batch in lines.chunks(batch_size) {
        let sender = sender.clone();
        let validators = Arc::clone(&validators);

        // Explicitly use rayon for parallel processing per batch
        thread_pool.scope(|s| {
            for (idx, line) in batch.iter().enumerate() {
                let sender = sender.clone();
                let validators = Arc::clone(&validators);
                let line_number = idx + 1; // Adjust if needed for global indexing

                s.spawn(move |_| {
                    for validator in validators.iter() {
                        validator.validate(line, line_number, &sender);
                    }
                });
            }
        });
    }

    // Drop the last sender explicitly
    drop(sender);

    // Explicitly collect and print issues
    receiver.iter().for_each(|issue| {
        println!(
            "[{}] Line {}, Position {:?}: {}",
            issue.validator, issue.line_number, issue.position, issue.message
        );
    });
}
