import csv
import sys

ILLEGAL_STRINGS = {r"137\n", "555555", "Zzzzz", "abcdef", "noway", "654321"}
EXPECTED_FIELDS = 50
MAX_LINE_LENGTH = 1024

def validate_illegal_chars(line: str, line_number: int) -> list:
    issues = []
    for illegal in ILLEGAL_STRINGS:
        if illegal in line:
            issues.append(f"[IllegalCharValidator] Line {line_number}: Contains illegal substring '{illegal}'")
    return issues

def validate_field_count(fields: list[str], line_number: int) -> list:
    if len(fields) != EXPECTED_FIELDS:
        return [f"[FieldCountValidator] Line {line_number}: Expected {EXPECTED_FIELDS} fields, found {len(fields)}"]
    return []

def validate_line_length(line: str, line_number: int) -> list:
    if len(line) > MAX_LINE_LENGTH:
        return [f"[LineLengthValidator] Line {line_number}: Line length {len(line)} exceeds maximum {MAX_LINE_LENGTH}"]
    return []

def main(csv_file_path) -> None:
    issues = []
    with open(csv_file_path, 'r', encoding='utf-8') as csvfile:
        reader = csv.reader(csvfile, delimiter=';', quotechar='"')

        for line_number, fields in enumerate(reader, start=1):
            line = ';'.join(fields)

            # Run validators
            issues.extend(validate_illegal_chars(line, line_number))
            issues.extend(validate_field_count(fields, line_number))
            issues.extend(validate_line_length(line, line_number))

    for issue in issues:
        print(issue)
    print(f"\nTotal issues found: {len(issues)}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python csv_validator.py <csv_file>")
        sys.exit(1)
    main(sys.argv[1])
