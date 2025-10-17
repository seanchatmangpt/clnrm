//! Acceptance tests for `clnrm dev --watch` command
//!
//! London School TDD approach:
//! - Start with user workflow (outside-in)
//! - Mock all collaborators (FileWatcher, Renderer, Executor)
//! - Verify interactions, not implementation

#[cfg(test)]
mod dev_watch_acceptance {
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use mockall::predicate::*;
    use mockall::mock;

    // Mock collaborators (defined by contracts)
    mock! {
        pub FileWatcher {}
        impl FileWatcher {
            fn watch(&self, path: &Path) -> Result<FileChangeReceiver, CleanroomError>;
            fn stop(&self) -> Result<(), CleanroomError>;
        }
    }

    mock! {
        pub TemplateRenderer {}
        impl TemplateRenderer {
            fn render_file(&self, path: &Path) -> Result<String, CleanroomError>;
        }
    }

    mock! {
        pub TestExecutor {}
        impl TestExecutor {
            fn run_test(&self, path: &Path) -> Result<TestResult, CleanroomError>;
        }
    }

    /// Acceptance Test: User runs `clnrm dev --watch tests/`
    /// Expected: Watch for file changes, re-render, re-run tests
    #[tokio::test]
    async fn acceptance_dev_watch_detects_changes_and_reruns() {
        // Arrange: User has test file that will be modified
        let test_file = PathBuf::from("tests/example.clnrm.toml");
        let watch_dir = PathBuf::from("tests/");

        let mut mock_watcher = MockFileWatcher::new();
        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_executor = MockTestExecutor::new();

        // User expectation: Watcher detects file change
        mock_watcher
            .expect_watch()
            .with(eq(watch_dir.as_path()))
            .times(1)
            .returning(move |_| {
                let (tx, rx) = tokio::sync::mpsc::channel(10);
                let test_file_clone = test_file.clone();

                // Simulate file change event after 100ms
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    tx.send(FileChangeEvent {
                        path: test_file_clone,
                        event_type: FileChangeType::Modified,
                    }).await.ok();
                });

                Ok(rx)
            });

        // User expectation: File is re-rendered when changed
        mock_renderer
            .expect_render_file()
            .with(eq(test_file.as_path()))
            .times(1)
            .returning(|_| {
                Ok(r#"[meta]
name = "example"
version = "0.1.0"
"#.to_string())
            });

        // User expectation: Test is re-run after render
        mock_executor
            .expect_run_test()
            .with(eq(test_file.as_path()))
            .times(1)
            .returning(|_| {
                Ok(TestResult {
                    passed: true,
                    duration_ms: 250,
                    error: None,
                })
            });

        // Act: User runs `clnrm dev --watch tests/`
        let dev_command = DevWatchCommand::new(
            Box::new(mock_watcher),
            Box::new(mock_renderer),
            Box::new(mock_executor),
        );

        let result = dev_command
            .run(&watch_dir, Duration::from_millis(200))
            .await;

        // Assert: User sees successful test re-run
        assert!(result.is_ok(), "Dev watch should succeed");

        // Mock expectations verify automatically on drop:
        // - watch() called once with correct path
        // - render_file() called once when file changed
        // - run_test() called once after render
    }

    /// Acceptance Test: User runs `clnrm dev --watch` with <3s cycle time
    /// Expected: Detect change, render, run within 3 seconds
    #[tokio::test]
    async fn acceptance_dev_watch_completes_cycle_under_3_seconds() {
        // Arrange
        let test_file = PathBuf::from("tests/fast.clnrm.toml");
        let watch_dir = PathBuf::from("tests/");

        let mut mock_watcher = MockFileWatcher::new();
        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_executor = MockTestExecutor::new();

        mock_watcher
            .expect_watch()
            .returning(move |_| {
                let (tx, rx) = tokio::sync::mpsc::channel(10);
                let test_file_clone = test_file.clone();

                tokio::spawn(async move {
                    // Immediate file change
                    tx.send(FileChangeEvent {
                        path: test_file_clone,
                        event_type: FileChangeType::Modified,
                    }).await.ok();
                });

                Ok(rx)
            });

        mock_renderer
            .expect_render_file()
            .returning(|_| Ok("rendered content".to_string()));

        mock_executor
            .expect_run_test()
            .returning(|_| {
                Ok(TestResult {
                    passed: true,
                    duration_ms: 100,
                    error: None,
                })
            });

        // Act: Measure cycle time
        let dev_command = DevWatchCommand::new(
            Box::new(mock_watcher),
            Box::new(mock_renderer),
            Box::new(mock_executor),
        );

        let start = std::time::Instant::now();
        let result = dev_command.run(&watch_dir, Duration::from_secs(3)).await;
        let cycle_time = start.elapsed();

        // Assert: Cycle completes in <3s
        assert!(result.is_ok());
        assert!(
            cycle_time < Duration::from_secs(3),
            "Dev watch cycle should complete in <3s, took {:?}",
            cycle_time
        );
    }

    /// Acceptance Test: User stops watching with Ctrl+C
    /// Expected: Graceful shutdown, cleanup
    #[tokio::test]
    async fn acceptance_dev_watch_stops_gracefully() {
        // Arrange
        let watch_dir = PathBuf::from("tests/");

        let mut mock_watcher = MockFileWatcher::new();
        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_executor = MockTestExecutor::new();

        mock_watcher
            .expect_watch()
            .returning(|_| {
                let (_tx, rx) = tokio::sync::mpsc::channel(10);
                Ok(rx)
            });

        // User expectation: stop() called on shutdown
        mock_watcher
            .expect_stop()
            .times(1)
            .returning(|| Ok(()));

        // Act: User runs watch, then stops
        let dev_command = DevWatchCommand::new(
            Box::new(mock_watcher),
            Box::new(mock_renderer),
            Box::new(mock_executor),
        );

        let handle = tokio::spawn(async move {
            dev_command.run(&watch_dir, Duration::from_secs(10)).await
        });

        // Simulate Ctrl+C after 100ms
        tokio::time::sleep(Duration::from_millis(100)).await;
        handle.abort();

        // Assert: Graceful shutdown (stop() called)
        // Mock expectation verified on drop
    }

    /// Acceptance Test: Debounce rapid file changes
    /// Expected: Only one re-run for multiple rapid changes
    #[tokio::test]
    async fn acceptance_dev_watch_debounces_rapid_changes() {
        // Arrange: Multiple rapid file changes
        let test_file = PathBuf::from("tests/example.clnrm.toml");
        let watch_dir = PathBuf::from("tests/");

        let mut mock_watcher = MockFileWatcher::new();
        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_executor = MockTestExecutor::new();

        mock_watcher
            .expect_watch()
            .returning(move |_| {
                let (tx, rx) = tokio::sync::mpsc::channel(10);
                let test_file_clone = test_file.clone();

                tokio::spawn(async move {
                    // Simulate 5 rapid changes within 200ms
                    for _ in 0..5 {
                        tx.send(FileChangeEvent {
                            path: test_file_clone.clone(),
                            event_type: FileChangeType::Modified,
                        }).await.ok();
                        tokio::time::sleep(Duration::from_millis(40)).await;
                    }
                });

                Ok(rx)
            });

        // User expectation: Only 1 render despite 5 changes (debounced)
        mock_renderer
            .expect_render_file()
            .times(1)
            .returning(|_| Ok("rendered".to_string()));

        // User expectation: Only 1 test run (debounced)
        mock_executor
            .expect_run_test()
            .times(1)
            .returning(|_| {
                Ok(TestResult {
                    passed: true,
                    duration_ms: 50,
                    error: None,
                })
            });

        // Act: Run with 300ms debounce
        let dev_command = DevWatchCommand::new_with_debounce(
            Box::new(mock_watcher),
            Box::new(mock_renderer),
            Box::new(mock_executor),
            Duration::from_millis(300),
        );

        let result = dev_command.run(&watch_dir, Duration::from_secs(1)).await;

        // Assert: Only one render + run despite multiple changes
        assert!(result.is_ok());
    }

    /// Acceptance Test: File watcher error recovery
    /// Expected: Graceful error handling, user sees helpful message
    #[tokio::test]
    async fn acceptance_dev_watch_handles_watcher_error() {
        // Arrange: Watcher fails (nonexistent directory)
        let watch_dir = PathBuf::from("nonexistent/");

        let mut mock_watcher = MockFileWatcher::new();
        let mock_renderer = MockTemplateRenderer::new();
        let mock_executor = MockTestExecutor::new();

        mock_watcher
            .expect_watch()
            .with(eq(watch_dir.as_path()))
            .times(1)
            .returning(|_| {
                Err(CleanroomError::validation_error(
                    "Directory does not exist: nonexistent/"
                ))
            });

        // Act: User runs watch on invalid path
        let dev_command = DevWatchCommand::new(
            Box::new(mock_watcher),
            Box::new(mock_renderer),
            Box::new(mock_executor),
        );

        let result = dev_command.run(&watch_dir, Duration::from_secs(1)).await;

        // Assert: User sees helpful error message
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("does not exist"));
    }

    // Supporting types (would be defined in contracts)
    struct FileChangeEvent {
        path: PathBuf,
        event_type: FileChangeType,
    }

    enum FileChangeType {
        Modified,
        Created,
        Deleted,
    }

    type FileChangeReceiver = tokio::sync::mpsc::Receiver<FileChangeEvent>;

    struct TestResult {
        passed: bool,
        duration_ms: u64,
        error: Option<String>,
    }

    struct CleanroomError {
        message: String,
    }

    impl CleanroomError {
        fn validation_error(msg: impl Into<String>) -> Self {
            Self { message: msg.into() }
        }
    }

    impl std::fmt::Display for CleanroomError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    // DevWatchCommand (to be implemented)
    struct DevWatchCommand {
        watcher: Box<dyn FileWatcher>,
        renderer: Box<dyn TemplateRenderer>,
        executor: Box<dyn TestExecutor>,
        debounce: Duration,
    }

    impl DevWatchCommand {
        fn new(
            watcher: Box<dyn FileWatcher>,
            renderer: Box<dyn TemplateRenderer>,
            executor: Box<dyn TestExecutor>,
        ) -> Self {
            Self {
                watcher,
                renderer,
                executor,
                debounce: Duration::from_millis(300),
            }
        }

        fn new_with_debounce(
            watcher: Box<dyn FileWatcher>,
            renderer: Box<dyn TemplateRenderer>,
            executor: Box<dyn TestExecutor>,
            debounce: Duration,
        ) -> Self {
            Self {
                watcher,
                renderer,
                executor,
                debounce,
            }
        }

        async fn run(&self, path: &Path, timeout: Duration) -> Result<(), CleanroomError> {
            // Implementation guided by these acceptance tests
            unimplemented!("DevWatchCommand::run - to be implemented via TDD")
        }
    }

    trait FileWatcher {
        fn watch(&self, path: &Path) -> Result<FileChangeReceiver, CleanroomError>;
        fn stop(&self) -> Result<(), CleanroomError>;
    }

    trait TemplateRenderer {
        fn render_file(&self, path: &Path) -> Result<String, CleanroomError>;
    }

    trait TestExecutor {
        fn run_test(&self, path: &Path) -> Result<TestResult, CleanroomError>;
    }
}
