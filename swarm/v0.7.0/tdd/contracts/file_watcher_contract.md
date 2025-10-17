# File Watcher Contract (London TDD)

## Interface Design (Outside-In)

The FileWatcher is a collaborator in the dev loop workflow. From the user's perspective:

```
User runs: clnrm dev --watch tests/
Expected: Watch for file changes, re-render, re-run tests automatically
```

## Mock Contract

```rust
pub trait FileWatcher: Send + Sync {
    /// Watch a directory for changes
    /// Returns a channel that emits changed file paths
    fn watch(&self, path: &Path) -> Result<FileChangeReceiver>;

    /// Stop watching all paths
    fn stop(&self) -> Result<()>;
}

pub struct FileChangeEvent {
    pub path: PathBuf,
    pub event_type: FileChangeType,
}

pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
}

pub type FileChangeReceiver = tokio::sync::mpsc::Receiver<FileChangeEvent>;
```

## Interaction Expectations (Behavior Verification)

### Scenario: User runs `clnrm dev --watch tests/`

```rust
#[tokio::test]
async fn test_dev_watch_detects_file_changes_and_reruns_tests() {
    // Arrange: Set up mock collaborators
    let mock_watcher = MockFileWatcher::new();
    let mock_renderer = MockTemplateRenderer::new();
    let mock_executor = MockTestExecutor::new();

    // Configure mock expectations
    mock_watcher.expect_watch()
        .with(eq(Path::new("tests/")))
        .returning(|_| {
            let (tx, rx) = tokio::sync::mpsc::channel(10);
            // Simulate file change after 100ms
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                tx.send(FileChangeEvent {
                    path: PathBuf::from("tests/example.clnrm.toml"),
                    event_type: FileChangeType::Modified,
                }).await.unwrap();
            });
            Ok(rx)
        });

    mock_renderer.expect_render_file()
        .with(eq(Path::new("tests/example.clnrm.toml")))
        .times(1)
        .returning(|_| Ok("rendered TOML content".to_string()));

    mock_executor.expect_run_test()
        .with(eq("tests/example.clnrm.toml"))
        .times(1)
        .returning(|_| Ok(TestResult::passed()));

    // Act: Run dev watch command
    let dev_command = DevWatchCommand::new(
        mock_watcher,
        mock_renderer,
        mock_executor,
    );

    let result = dev_command.run("tests/", Duration::from_millis(200)).await;

    // Assert: Verify interaction sequence
    assert!(result.is_ok());
    // Mock expectations verify automatically on drop
}
```

## Critical Interaction Sequence

1. User → DevWatchCommand: run("tests/")
2. DevWatchCommand → FileWatcher: watch("tests/")
3. FileWatcher → DevWatchCommand: FileChangeEvent(Modified, "tests/example.clnrm.toml")
4. DevWatchCommand → TemplateRenderer: render_file("tests/example.clnrm.toml")
5. DevWatchCommand → TestExecutor: run_test("tests/example.clnrm.toml")
6. DevWatchCommand → User: Display results

## Performance Contract

- File change detection: &lt;100ms from FS event to detection
- Re-render + re-run: &lt;3s total (80/20 target)
- Debounce rapid changes: 200ms window

## Error Scenarios

### Invalid file path
```rust
mock_watcher.expect_watch()
    .with(eq(Path::new("nonexistent/")))
    .returning(|_| Err(CleanroomError::validation_error("Path does not exist")));
```

### File watcher crashes
```rust
mock_watcher.expect_watch()
    .returning(|_| {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            tx.send(FileChangeEvent::error("Watcher crashed")).await.ok();
        });
        Ok(rx)
    });
```

## Implementation Notes

- Use `notify` crate for cross-platform file watching
- Debounce with `tokio::time::sleep`
- Mock should support both synchronous setup and async event emission
- All trait methods MUST be sync (use `block_in_place` internally)
