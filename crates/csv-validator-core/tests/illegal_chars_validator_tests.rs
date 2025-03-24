use csv_validator_core::{IllegalCharacterValidator, issue::ValidationContext, Validator};
use pretty_assertions::assert_eq;

#[test]
fn illegal_character_validator_check_only_mode() {
    let validator = IllegalCharacterValidator::new(&["@"], &["_"]);
    let mut context = ValidationContext::new();

    let input = "hello@world";
    let result = validator.validate(input, &mut context, 1, false);

    assert_eq!(context.issues.len(), 1);
    assert_eq!(result.line, "hello@world");
    assert!(!result.modified);
}

#[test]
fn illegal_character_validator_fix_mode() {
    let validator = IllegalCharacterValidator::new(&["@"], &["_"]);
    let mut context = ValidationContext::new();

    let input = "hello@world";
    let result = validator.validate(input, &mut context, 1, true);

    assert_eq!(context.issues.len(), 1);
    assert_eq!(result.line, "hello_world");
    assert!(result.modified);
}

#[test]
fn validator_chaining_example() {
    let illegal_char_validator = IllegalCharacterValidator::new(&["@"], &["_"]);
    let another_illegal_validator = IllegalCharacterValidator::new(&["!"], &["."]);
    let validators: Vec<&dyn Validator> = vec![&illegal_char_validator, &another_illegal_validator];

    let mut context = ValidationContext::new();
    let line_number = 1;
    let fix = true;

    let original_line = "hello@world!";

    // Start chaining
    let mut result = ValidationResult::new(original_line);

    for validator in validators {
        result = validator.validate(result, &mut context, line_number, fix);
    }

    assert_eq!(result.line, "hello_world.");
    assert_eq!(context.issues.len(), 2);
}