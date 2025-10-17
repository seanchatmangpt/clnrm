# Formatter Contract (London TDD)

## Interface Design (Outside-In)

The Formatter is a collaborator that formats Tera templates and TOML files. From the user's perspective:

```
User runs: clnrm fmt tests/example.clnrm.toml
Expected: Format template + TOML, preserve semantics, be idempotent
```

## Mock Contract

```rust
pub trait Formatter: Send + Sync {
    /// Format a file in-place or check formatting
    fn format(&self, path: &Path, options: FormatOptions) -> Result<FormatResult>;

    /// Verify idempotency (format twice == format once)
    fn verify_idempotent(&self, path: &Path) -> Result<bool>;
}

pub struct FormatOptions {
    pub check_only: bool,      // Don't modify, just check
    pub verify_idempotency: bool, // Verify format is stable
    pub preserve_comments: bool,
}

pub struct FormatResult {
    pub file_path: PathBuf,
    pub changed: bool,
    pub original_content: String,
    pub formatted_content: String,
    pub idempotent: Option<bool>, // Only if verify_idempotency = true
}
```

## Interaction Expectations (Behavior Verification)

### Scenario: User runs `clnrm fmt tests/example.clnrm.toml`

```rust
#[test]
fn test_fmt_formats_file_and_writes_result() {
    // Arrange: Set up mock collaborators
    let mock_reader = MockFileReader::new();
    let mock_formatter = MockTomlFormatter::new();
    let mock_writer = MockFileWriter::new();

    // Configure mock expectations
    mock_reader.expect_read()
        .with(eq(Path::new("tests/example.clnrm.toml")))
        .times(1)
        .returning(|_| Ok(r#"[meta]
name="unformatted"
version = "0.1.0"

[[scenario]]
name     =    "test"
"#.to_string()));

    mock_formatter.expect_format_toml()
        .times(1)
        .returning(|content| Ok(r#"[meta]
name = "unformatted"
version = "0.1.0"

[[scenario]]
name = "test"
"#.to_string()));

    mock_writer.expect_write()
        .with(eq(Path::new("tests/example.clnrm.toml")), any())
        .times(1)
        .returning(|_, _| Ok(()));

    // Act: Run format command
    let fmt_command = FmtCommand::new(
        mock_reader,
        mock_formatter,
        mock_writer,
    );

    let result = fmt_command.run(&[PathBuf::from("tests/example.clnrm.toml")], FormatOptions {
        check_only: false,
        verify_idempotency: false,
        preserve_comments: true,
    });

    // Assert: Verify interaction sequence
    assert!(result.is_ok());
    let format_results = result.unwrap();
    assert_eq!(format_results.len(), 1);
    assert!(format_results[0].changed);
}
```

### Scenario: Verify idempotency

```rust
#[test]
fn test_fmt_verify_idempotency_succeeds() {
    // Arrange
    let mock_reader = MockFileReader::new();
    let mock_formatter = MockTomlFormatter::new();
    let mock_writer = MockFileWriter::new();

    let formatted = r#"[meta]
name = "test"
version = "0.1.0"
"#;

    // First read: original content
    mock_reader.expect_read()
        .times(1)
        .returning(move |_| Ok(formatted.to_string()));

    // First format
    mock_formatter.expect_format_toml()
        .times(1)
        .returning(move |_| Ok(formatted.to_string()));

    // Second format (verify idempotency)
    mock_formatter.expect_format_toml()
        .times(1)
        .returning(move |_| Ok(formatted.to_string()));

    // No write in check-only mode
    mock_writer.expect_write()
        .times(0);

    // Act
    let fmt_command = FmtCommand::new(mock_reader, mock_formatter, mock_writer);
    let result = fmt_command.run(&[PathBuf::from("test.toml")], FormatOptions {
        check_only: false,
        verify_idempotency: true,
        preserve_comments: true,
    });

    // Assert: Idempotency verified
    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results[0].idempotent, Some(true));
}
```

### Scenario: Check-only mode (no writes)

```rust
#[test]
fn test_fmt_check_only_does_not_write() {
    // Arrange
    let mock_reader = MockFileReader::new();
    let mock_formatter = MockTomlFormatter::new();
    let mock_writer = MockFileWriter::new();

    mock_reader.expect_read()
        .returning(|_| Ok("[meta]\nname=\"bad\"".to_string()));

    mock_formatter.expect_format_toml()
        .returning(|_| Ok("[meta]\nname = \"bad\"".to_string()));

    // CRITICAL: Writer should NOT be called in check-only mode
    mock_writer.expect_write()
        .times(0);

    // Act
    let fmt_command = FmtCommand::new(mock_reader, mock_formatter, mock_writer);
    let result = fmt_command.run(&[PathBuf::from("test.toml")], FormatOptions {
        check_only: true,
        verify_idempotency: false,
        preserve_comments: true,
    });

    // Assert
    assert!(result.is_ok());
    assert!(result.unwrap()[0].changed);
}
```

## Critical Interaction Sequence

1. User → FmtCommand: run(files, options)
2. FmtCommand → FileReader: read(file_path)
3. FmtCommand → TomlFormatter: format_toml(content)
4. **If check_only = false**: FmtCommand → FileWriter: write(file_path, formatted)
5. **If verify_idempotency = true**: FmtCommand → TomlFormatter: format_toml(formatted) again
6. FmtCommand → User: Display FormatResult

## Formatting Rules (80/20 Priority)

### MUST apply:
1. Normalize spacing around `=` (key = value)
2. Consistent table header formatting ([meta], [[scenario]])
3. Remove trailing whitespace
4. Ensure newline at EOF
5. Consistent quote style (prefer double quotes)

### SHOULD apply:
1. Alphabetize keys within sections
2. Consistent array formatting
3. Align multi-line values

## Performance Contract

- Format typical test file: &lt;100ms
- Idempotency check: &lt;200ms (2x format)
- No Docker operations
- Atomic file writes (write to temp, then rename)

## Error Scenarios

### Invalid TOML syntax
```rust
mock_formatter.expect_format_toml()
    .returning(|_| Err(CleanroomError::validation_error("Cannot format invalid TOML")));
```

### File write fails (permissions)
```rust
mock_writer.expect_write()
    .returning(|_, _| Err(CleanroomError::io_error("Permission denied")));
```

### Idempotency failure
```rust
mock_formatter.expect_format_toml()
    .times(2)
    .returning(call_count, |_| {
        if call_count == 0 {
            Ok("version = \"0.1.0\"".to_string())
        } else {
            Ok("version=\"0.1.0\"".to_string()) // Different!
        }
    });
// Should detect and error
```

## Implementation Notes

- Use `toml-edit` crate for semantics-preserving formatting
- Formatter should be pure function (no side effects)
- FileWriter should use atomic operations (temp file + rename)
- Idempotency check: format(format(x)) == format(x)
- All trait methods MUST be sync
