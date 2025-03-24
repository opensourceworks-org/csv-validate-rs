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
