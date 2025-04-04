// use clap::{Parser, Args};
// use std::fs::File;
// use std::io::{self, Read, Write};
// use std::sync::Arc;
// use crossbeam_channel::unbounded;
// use rayon::ThreadPoolBuilder;
// use csv_validator_core::{Validator, ValidatorSpec, ValidationOptions, validate_file, ValidationIssue, OptimizedQuoteAwareReader, execute_validators};
//
//
// #[derive(Parser, Debug)]
// #[command(version = "1.0", about = "High-performance CSV validator")]
// struct CliArgs {
//     /// Optional input file (defaults to stdin if not provided)
//     #[arg(value_name = "FILE")]
//     input: Option<String>,
//
//     /// Use YAML config file instead of CLI flags
//     #[arg(long, value_name = "YAML")]
//     config: Option<String>,
//
//     /// CSV delimiter
//     #[arg(long, default_value = ",")]
//     separator: char,
//
//     /// Output file (defaults to stdout)
//     #[arg(short, long)]
//     output: Option<String>,
//
//     /// Number of threads
//     #[arg(short, long, default_value_t = num_cpus::get())]
//     threads: usize,
//
//     /// Batch size
//     #[arg(short, long, default_value_t = 100_000)]
//     batch_size: usize,
//
//     #[command(flatten)]
//     validator: ValidatorKind,
// }
//
// #[derive(Args, Debug)]
// #[group(required = true, multiple = false)]
// pub struct ValidatorKind {
//     #[arg(long)]
//     illegal_chars: Option<String>,
//
//     #[arg(long)]
//     field_count: Option<usize>,
//
//     #[arg(long)]
//     max_line_length: Option<usize>,
// }
//
// impl ValidatorKind {
//     pub fn into_spec(self) -> ValidatorSpec {
//         match self {
//             ValidatorKind { illegal_chars: Some(chars), .. } => {
//                 ValidatorSpec::IllegalChars {
//                     illegal_chars: chars.split(',').map(str::to_string).collect(),
//                     enabled: true,
//                 }
//             }
//             ValidatorKind { field_count: Some(expected), .. } => {
//                 ValidatorSpec::FieldCount {
//                     expected,
//                     enabled: true,
//                 }
//             }
//             ValidatorKind { max_line_length: Some(max), .. } => {
//                 ValidatorSpec::LineLength {
//                     max_length: max,
//                     enabled: true,
//                 }
//             }
//             _ => panic!("No validator specified"),
//         }
//     }
// }
//
// fn main() -> anyhow::Result<()> {
//     let args = CliArgs::parse();
//     let separator = args.separator as u8;
//     let buffer_size = 8 * 1024 * 1024;
//
//     // Read input path
//     let input_path = args.input.clone().unwrap_or_else(|| "-".to_string());
//
//
//
//     // Build the validator
//     let spec = args.validator.into_spec();
//     let validator = spec.into_validator(separator);
//     let validators = Arc::new(vec![validator]);
//
//     let options = ValidationOptions {
//         threads: args.threads,
//         batch_size: args.batch_size,
//         buffer_size,
//     };
//
//     let issues = match args.input {
//         Some(input_path) => validate_file(&input_path, validators, options)?,
//         None => {
//                 let mut reader = OptimizedQuoteAwareReader::new(input, buffer_capacity);
//                 //
//                 // let validator: Box<dyn Validator> = build_validator_from_args(&args.validator, separator);
//                 // let validators = Arc::new(vec![validator]);
//
//                 let thread_pool = ThreadPoolBuilder::new()
//                     .num_threads(args.threads)
//                     .build()?;
//
//                 let (sender, receiver) = unbounded();
//
//                 let mut batch = Vec::with_capacity(args.batch_size);
//                 let mut line_buf = Vec::with_capacity(1024);
//                 let mut line_number = 0;
//
//                 while let Some(line) = reader.next_logical_line(&mut line_buf)? {
//                     line_number += 1;
//                     batch.push((line_number, line.to_vec()));
//
//                     if batch.len() >= args.batch_size {
//                         execute_validators(batch.drain(..).collect(), validators.clone(), sender.clone());
//                     }
//                 }
//
//                 if !batch.is_empty() {
//                     execute_validators(batch.drain(..).collect(), validators.clone(), sender.clone());
//                 }
//
//                 drop(sender);
//
//                 for batched_issues in receiver.iter() {
//
//                 }
//
//                 vec![]
//         }
//     }
//
//
//     let mut writer: Box<dyn Write> = match args.output.as_deref() {
//         Some("-") | None => Box::new(io::stdout()),
//         Some(path) => Box::new(File::create(path)?),
//     };
//
//     for issue in issues {
//         writeln!(
//             writer,
//             "[{}] Line {}, Position {:?}: {}",
//             issue.validator, issue.line_number, issue.position, issue.message
//         )?;
//     }
//
//     Ok(())
// }
//


use clap::{Parser, Args};
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::Arc;
use crossbeam_channel::unbounded;
use rayon::ThreadPoolBuilder;

use csv_validator_core::{
    Validator,
    IllegalCharactersValidator,
    FieldCountValidator,
    LineLengthValidator,
    OptimizedQuoteAwareReader,
    execute_validators,
    ValidationIssue,
};

pub mod config;


#[derive(Parser, Debug)]
#[command(version = "1.0", about = "High-performance CSV validator")]
struct CliArgs {
    /// Optional input file (defaults to stdin if not provided)
    #[arg(value_name = "FILE")]
    input: Option<String>,

    /// Use YAML config file instead of CLI flags
    #[arg(long, value_name = "YAML")]
    config: Option<String>,

    /// CSV delimiter
    #[arg(long, default_value = ",")]
    separator: char,

    /// Output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Number of threads
    #[arg(short, long, default_value_t = num_cpus::get())]
    threads: usize,

    /// Batch size
    #[arg(short, long, default_value_t = 100_000)]
    batch_size: usize,

    #[command(flatten)]
    validator: ValidatorKind,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct ValidatorKind {
    /// Comma-separated list of illegal characters
    #[arg(long)]
    illegal_chars: Option<String>,

    /// Expected number of fields
    #[arg(long)]
    field_count: Option<usize>,

    /// Maximum line length
    #[arg(long)]
    max_line_length: Option<usize>,
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let separator = args.separator as u8;
    let buffer_capacity = 8 * 1024 * 1024;

    let input: Box<dyn Read> = match args.input.as_deref() {
        Some("-") | None => Box::new(io::stdin()),
        Some(path) => Box::new(File::open(path)?),
    };

    let threads = &args.threads;
    dbg!(&threads);

    let mut reader = OptimizedQuoteAwareReader::new(input, buffer_capacity);

    let validator: Box<dyn Validator> = build_validator_from_args(&args.validator, separator);
    let validators = Arc::new(vec![validator]);

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build()?;

    let (sender, receiver) = unbounded();

    let mut batch = Vec::with_capacity(args.batch_size);
    let mut line_buf = Vec::with_capacity(1024);
    let mut line_number = 0;

    while let Some(line) = reader.next_logical_line(&mut line_buf)? {
        line_number += 1;
        batch.push((line_number, line.to_vec()));

        if batch.len() >= args.batch_size {
            execute_validators(batch.drain(..).collect(), validators.clone(), sender.clone());
        }
    }

    if !batch.is_empty() {
        execute_validators(batch.drain(..).collect(), validators.clone(), sender.clone());
    }

    drop(sender);

    let mut writer: Box<dyn Write> = match args.output.as_deref() {
        Some("-") | None => Box::new(io::stdout()),
        Some(path) => Box::new(File::create(path)?),
    };

    for issues in receiver.iter() {
        for issue in issues {
            writeln!(
                writer,
                "[{}] Line {}, Position {:?}: {}",
                issue.validator, issue.line_number, issue.position, issue.message
            )?;
        }
    }

    Ok(())
}

fn build_validator_from_args(kind: &ValidatorKind, separator: u8) -> Box<dyn Validator> {
    if let Some(chars) = &kind.illegal_chars {
        let list: Vec<&str> = chars.split(',').map(str::trim).collect();
        Box::new(IllegalCharactersValidator::new( &list))
    } else if let Some(count) = kind.field_count {
        Box::new(FieldCountValidator::new( count, separator))
    } else if let Some(max) = kind.max_line_length {
        Box::new(LineLengthValidator::new( max))
    } else {
        panic!("No validator specified")
    }
}
