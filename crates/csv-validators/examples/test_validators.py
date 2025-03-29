from csv_validators import (
    validate_file_py,
    PyValidatorSpec,
    PyValidationOptions
)

def test_illegal_chars():
    path = "../csv-validator-core/examples/output_2g.csv"

    # Create validator
    validators = [
        PyValidatorSpec.illegal_chars([r"137\n", '555555', 'Zzzzz', "abcdef", "noway", "654321"])
    ]

    # Optional: define options
    options = PyValidationOptions()
    options.threads = 8
    options.batch_size = 100_000
    options.buffer_size = 64 * 1024 * 1024

    issues = validate_file_py(path, validators, options)

    for issue in issues:
        print(
            f"[{issue.validator}] Line {issue.line_number}, "
            f"Pos {issue.position}: {issue.message}"
        )

    assert len(issues) > 0, "Expected at least one validation issue"

if __name__ == "__main__":
    test_illegal_chars()
