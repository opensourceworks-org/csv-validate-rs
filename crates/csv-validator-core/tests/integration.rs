use csv_validator_core::{
    ValidatorSpec, ValidationOptions, validate_file,
};
use std::sync::Arc;
use std::path::Path;

#[test]
fn test_validate_bad_csv_file() {
    let path = "tests/data/bad.csv";

    let specs = vec![
        ValidatorSpec::new_illegal_chars(vec![
            "@@".to_string(),
            "Zzzzz".to_string(),
        ]),
        ValidatorSpec::new_field_count(3),
        ValidatorSpec::LineLength {
            enabled: true,
            max_length: 80,
        },
    ];

    let validators = Arc::new(
        specs.into_iter()
            .map(|s| s.into_validator(b','))
            .collect::<Vec<_>>(),
    );

    let options = ValidationOptions {
        threads: 2,
        batch_size: 100,
        buffer_size: 1024 * 1024,
        preserve_order: false,
    };

    let issues = validate_file(path, validators, options).expect("validation failed");

    assert!(!issues.is_empty(), "Expected validation issues");

    for issue in &issues {
        println!(
            "[{}] Line {}, Pos {:?}: {}",
            issue.validator, issue.line_number, issue.position, issue.message
        );
    }

    // Example: assert that line 5 contains illegal Zzzzz
    assert!(issues.iter().any(|i| i.line_number == 5 && i.message.contains("Zzzzz")));
}
