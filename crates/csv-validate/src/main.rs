use clap::Parser;
use csv_validator_core::{
    IllegalCharactersValidator, FieldCountValidator, LineLengthValidator,
    Validator, ValidationIssue, OptimizedQuoteAwareReader, execute_validators,
};
use crossbeam_channel::unbounded;
use rayon::ThreadPoolBuilder;
use std::{sync::Arc, io::Write, fs};
use std::fs::File;
use serde::Deserialize;
use crate::config::{build_validator_from_cli, build_validators_from_config, load_config};
use std::io;
use std::io::Read;

pub mod config;

/// CLI for CSV Validator
#[derive(Parser, Debug)]
#[command(version = "1.0", about = "High-performance CSV validator")]
struct CliArgs {
    /// Input file path ('-' for stdin)
    #[arg(short, long, default_value = "-")]
    input: String,

    /// Output file path ('-' for stdout)
    #[arg(short, long, default_value = "-")]
    output: String,

    /// Config file (YAML format)
    #[arg(short, long)]
    config: Option<String>,

    /// Fallback: Single validator type
    #[arg(long)]
    validator: Option<String>,

    /// Illegal characters (for CLI mode)
    #[arg(long)]
    illegal_chars: Option<String>,

    /// Expected field count (for CLI mode)
    #[arg(long)]
    field_count: Option<usize>,

    /// Max line length (for CLI mode)
    #[arg(long, default_value_t = 1024)]
    max_line_length: usize,

    /// Separator (for CLI mode)
    #[arg(long, default_value = ",")]
    separator: char,

    /// Threads
    #[arg(short, long, default_value_t = 4)]
    threads: usize,

    /// Batch size
    #[arg(short, long, default_value_t = 100_000)]
    batch_size: usize,
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let buffer_capacity = 8 * 1024 * 1024;

    let validators = if let Some(config_path) = args.config.clone() {
        Arc::new(build_validators_from_config(load_config(config_path)?))
    } else {
        Arc::new(build_validator_from_cli(&args)?)
    };

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build()?;

    let (sender, receiver) = unbounded();

    let input: Box<dyn Read> = match args.input.as_str() {
        "-" => Box::new(io::stdin()),
        path => Box::new(File::open(path)?),
    };

    let mut reader = OptimizedQuoteAwareReader::new(input, buffer_capacity);


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

    let mut writer: Box<dyn Write> = if args.output == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(fs::File::create(&args.output)?)
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
