# Code Standards

## Style Standards
- **Formatting**: `cargo fmt` must pass - enforced in CI
- **Linting**: `cargo clippy` must pass with no warnings - enforced in CI
- **Imports**: Alphabetical order within groups (std, external, internal)
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types

## Error Handling Standards
- **No unwrap/expect**: Production code must use `Result<T, E>` types
- **Consistent error types**: Use `CleanroomError` enum with context methods
- **Error propagation**: Use `?` operator for clean error handling
- **Test code**: May use unwrap/expect for simplicity but document why

## Quality Standards
- **Test coverage**: Minimum 80% coverage for all modules
- **Error handling**: All fallible operations return `Result`
- **Documentation**: All public APIs must be documented
- **Complexity**: Functions should be focused and testable

## Pattern Standards
- **Function organization**: Public functions first, private functions second
- **Error handling**: Consistent patterns using `Result<T, CleanroomError>`
- **Validation**: Use type-level validation (Poka-yoke) when possible
- **Testing**: AAA pattern (Arrange, Act, Assert) in all tests

## Documentation Standards
- **Public APIs**: All public functions/structs/traits must have doc comments
- **Examples**: Include usage examples where helpful
- **Error conditions**: Document when functions can fail
- **Thread safety**: Document thread safety guarantees
