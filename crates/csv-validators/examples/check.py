import sys

def validate_csv(file_path, illegal_chars=(r"137\n", '555555', 'Zzzzz', "abcdef", "noway", "654321")):
    buffer_size = 1024 * 1024  # Explicitly 1MB buffer
    line_number = 0
    issues = []

    with open(file_path, 'r', encoding='utf-8', buffering=buffer_size) as file:
        for line in file:
            line_number += 1
            line = line.rstrip('\n')

            for char in illegal_chars:
                pos = line.find(char)
                if pos != -1:
                    issues.append((line_number, pos, char))

    return issues

def main():
    if len(sys.argv) != 2:
        print("Usage: python validate_csv.py <csv_file>")
        sys.exit(1)

    file_path = sys.argv[1]
    issues = validate_csv(file_path)

    for line_number, pos, char in issues:
        print(f"[IllegalCharValidator] Line {line_number}, Position {pos}: Illegal character '{char}'")

    print(f"\nTotal issues found: {len(issues)}")

if __name__ == '__main__':
    main()
