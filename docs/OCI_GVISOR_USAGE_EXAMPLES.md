# OCI + gVisor Usage Examples

## Installation

### 1. Install gVisor runsc

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y software-properties-common
sudo add-apt-repository ppa:gvisor/gvisor
sudo apt-get install -y runsc
runsc --version
```

**From Binary:**
```bash
ARCH=$(uname -m)
wget https://storage.googleapis.com/gvisor/releases/release/latest/${ARCH}/runsc
chmod +x runsc
sudo mv runsc /usr/local/bin/
runsc --version
```

### 2. Verify Installation

```bash
# Check runsc is available
which runsc

# Test basic functionality
runsc --version
```

## Basic Usage

### Example 1: Hello World with Alpine

```rust
use clnrm_core::backend::{Backend, Cmd, GvisorBackend};
use clnrm_core::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create gVisor backend with Alpine image
    let backend = GvisorBackend::new("alpine:latest").await?;

    // Create command
    let cmd = Cmd::new("echo").arg("Hello from gVisor!");

    // Execute
    let result = backend.run_cmd(cmd)?;

    println!("Exit code: {}", result.exit_code);
    println!("Output: {}", result.stdout);
    println!("Duration: {}ms", result.duration_ms);

    Ok(())
}
```

**Output:**
```
Loading OCI image
Pulling image from registry: registry-1.docker.io/library/alpine:latest
Fetched manifest with 1 layers
Image loaded: 1 layers, amd64 architecture
Creating OCI bundle
Rootfs extracted to: /tmp/bundles/abc-123/rootfs
Bundle created: /tmp/bundles/abc-123
Starting container clnrm-abc-123 with runsc
Container execution complete: exit code 0
Exit code: 0
Output: Hello from gVisor!
Duration: 2341ms
```

### Example 2: Multi-Step Scenario

```rust
use clnrm::scenario;

#[tokio::main]
async fn main() -> Result<()> {
    let scenario = scenario("integration_test")
        .step("setup".to_string(), ["apk", "add", "--no-cache", "curl"])
        .step("test".to_string(), ["sh", "-c", "echo 'Testing' | tee /tmp/test.txt"])
        .step("verify".to_string(), ["cat", "/tmp/test.txt"]);

    let result = scenario.run_gvisor("alpine:latest").await?;

    println!("All steps completed in {}ms", result.duration_ms);
    println!("Final output:\n{}", result.stdout);

    Ok(())
}
```

### Example 3: Custom Image

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Use custom registry image
    let backend = GvisorBackend::new("myregistry.io/myapp:v1.2.3").await?;

    let cmd = Cmd::new("/app/myapp")
        .arg("--config")
        .arg("/etc/config.yaml")
        .env("LOG_LEVEL", "debug");

    let result = backend.run_cmd(cmd)?;

    if result.exit_code == 0 {
        println!("Application executed successfully");
    } else {
        eprintln!("Application failed: {}", result.stderr);
    }

    Ok(())
}
```

## Advanced Usage

### Example 4: With Environment Variables

```rust
use clnrm_core::backend::{Backend, Cmd, GvisorBackend};

#[tokio::main]
async fn main() -> Result<()> {
    let backend = GvisorBackend::new("alpine:latest").await?;

    let cmd = Cmd::new("sh")
        .args(&["-c", "echo $MESSAGE"])
        .env("MESSAGE", "Hello from environment!")
        .env("DEBUG", "1");

    let result = backend.run_cmd(cmd)?;
    println!("Output: {}", result.stdout);

    Ok(())
}
```

### Example 5: With Working Directory

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let backend = GvisorBackend::new("ubuntu:22.04").await?;

    let cmd = Cmd::new("pwd")
        .workdir("/tmp".into());

    let result = backend.run_cmd(cmd)?;
    assert!(result.stdout.trim() == "/tmp");

    Ok(())
}
```

### Example 6: With Timeout

```rust
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let backend = GvisorBackend::new("alpine:latest")
        .await?
        .with_timeout(Duration::from_secs(5));

    let cmd = Cmd::new("sleep").arg("10");

    match backend.run_cmd(cmd) {
        Ok(_) => println!("Completed"),
        Err(e) => println!("Timed out: {}", e),
    }

    Ok(())
}
```

### Example 7: With Policy

```rust
use clnrm_core::policy::{Policy, SecurityLevel};

#[tokio::main]
async fn main() -> Result<()> {
    let policy = Policy::with_security_level(SecurityLevel::High);

    let backend = GvisorBackend::new("alpine:latest")
        .await?
        .with_policy(policy);

    let cmd = Cmd::new("echo").arg("Secure execution");

    let result = backend.run_cmd(cmd)?;
    println!("Output: {}", result.stdout);

    Ok(())
}
```

## Database Testing

### Example 8: SurrealDB Integration

```rust
use clnrm_core::backend::{Backend, Cmd, GvisorBackend};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Start SurrealDB in background
    let backend = GvisorBackend::new("surrealdb/surrealdb:latest")
        .await?
        .with_timeout(Duration::from_secs(30));

    let start_cmd = Cmd::new("surreal")
        .args(&["start", "--bind", "0.0.0.0:8000", "memory"]);

    // Note: This will timeout since SurrealDB runs continuously
    // In real usage, you'd run this in background and connect to it
    match backend.run_cmd(start_cmd) {
        Ok(_) => println!("Database ready"),
        Err(e) => println!("Expected timeout: {}", e),
    }

    Ok(())
}
```

### Example 9: PostgreSQL Tests

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let backend = GvisorBackend::new("postgres:15").await?;

    // Initialize database
    let init_cmd = Cmd::new("psql")
        .args(&["-c", "CREATE DATABASE test;"])
        .env("POSTGRES_PASSWORD", "testpass");

    let result = backend.run_cmd(init_cmd)?;

    if result.exit_code == 0 {
        println!("Database initialized");

        // Run tests
        let test_cmd = Cmd::new("psql")
            .args(&["-d", "test", "-c", "SELECT 1;"])
            .env("POSTGRES_PASSWORD", "testpass");

        let test_result = backend.run_cmd(test_cmd)?;
        println!("Test output: {}", test_result.stdout);
    }

    Ok(())
}
```

## Image Caching

### Example 10: Manual Cache Management

```rust
use clnrm_core::backend::oci::ImageCache;

#[tokio::main]
async fn main() -> Result<()> {
    // Create cache with 5GB limit
    let cache = ImageCache::new(5)?;

    // Check if image is cached
    if let Some(image) = cache.get("alpine:latest").await? {
        println!("Image cached: {} layers", image.layers.len());
    } else {
        println!("Image not in cache");
    }

    // Clear entire cache
    cache.clear().await?;
    println!("Cache cleared");

    Ok(())
}
```

### Example 11: Pre-warming Cache

```rust
use clnrm_core::backend::{GvisorBackend, OciImageLoader, ImageSource};

#[tokio::main]
async fn main() -> Result<()> {
    let loader = OciImageLoader::new()?;

    // Pre-load common images
    let images = vec![
        "alpine:latest",
        "ubuntu:22.04",
        "surrealdb/surrealdb:latest",
    ];

    for image_ref in images {
        println!("Pre-loading: {}", image_ref);

        let source = ImageSource::Registry {
            registry: "registry-1.docker.io".to_string(),
            repository: format!("library/{}", image_ref.split(':').next().unwrap()),
            tag: image_ref.split(':').nth(1).unwrap_or("latest").to_string(),
        };

        loader.load_image(source).await?;
        println!("  ✓ Loaded");
    }

    println!("Cache warmed!");

    Ok(())
}
```

## Error Handling

### Example 12: Comprehensive Error Handling

```rust
use clnrm_core::backend::{Backend, Cmd, GvisorBackend};
use clnrm_core::error::{CleanroomError, ErrorKind};

#[tokio::main]
async fn main() {
    match run_test().await {
        Ok(result) => {
            println!("Success: {}", result.stdout);
        }
        Err(e) => {
            eprintln!("Error: {}", e);

            match e.kind {
                ErrorKind::NetworkError => {
                    eprintln!("Network issue - check connectivity");
                }
                ErrorKind::ContainerError => {
                    eprintln!("Container issue - check runsc installation");
                }
                ErrorKind::Timeout => {
                    eprintln!("Timeout - increase timeout or optimize command");
                }
                _ => {
                    eprintln!("Other error: {:?}", e.kind);
                }
            }
        }
    }
}

async fn run_test() -> Result<RunResult> {
    let backend = GvisorBackend::new("alpine:latest").await?;
    let cmd = Cmd::new("echo").arg("test");
    backend.run_cmd(cmd)
}
```

### Example 13: Retry Logic

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let max_retries = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;

        match try_operation().await {
            Ok(result) => {
                println!("Success on attempt {}", attempt);
                return Ok(());
            }
            Err(e) if attempt < max_retries => {
                eprintln!("Attempt {} failed: {}", attempt, e);
                eprintln!("Retrying in 2s...");
                sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

async fn try_operation() -> Result<RunResult> {
    let backend = GvisorBackend::new("alpine:latest").await?;
    let cmd = Cmd::new("echo").arg("test");
    backend.run_cmd(cmd)
}
```

## Testing Patterns

### Example 14: Integration Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires runsc installed
    async fn test_gvisor_alpine_execution() {
        let backend = GvisorBackend::new("alpine:latest")
            .await
            .expect("Failed to create backend");

        let cmd = Cmd::new("echo").arg("integration test");

        let result = backend.run_cmd(cmd).expect("Failed to run command");

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("integration test"));
        assert!(result.duration_ms > 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_gvisor_command_failure() {
        let backend = GvisorBackend::new("alpine:latest").await.unwrap();

        let cmd = Cmd::new("false"); // Always fails

        let result = backend.run_cmd(cmd).unwrap();

        assert_ne!(result.exit_code, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_gvisor_caching() {
        // First run (pulls image)
        let start1 = std::time::Instant::now();
        let backend1 = GvisorBackend::new("alpine:latest").await.unwrap();
        let duration1 = start1.elapsed();

        // Second run (uses cache)
        let start2 = std::time::Instant::now();
        let backend2 = GvisorBackend::new("alpine:latest").await.unwrap();
        let duration2 = start2.elapsed();

        // Cached should be faster
        assert!(duration2 < duration1);
        println!("First: {:?}, Cached: {:?}", duration1, duration2);
    }
}
```

## Performance Optimization

### Example 15: Parallel Test Execution

```rust
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    let tests = vec![
        "test1: echo first",
        "test2: echo second",
        "test3: echo third",
    ];

    let mut handles = Vec::new();

    for test in tests {
        let handle = task::spawn(async move {
            let backend = GvisorBackend::new("alpine:latest").await?;
            let parts: Vec<&str> = test.split(": ").collect();
            let cmd = Cmd::new("echo").arg(parts[1]);
            backend.run_cmd(cmd)
        });

        handles.push(handle);
    }

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(Ok(result)) => {
                println!("Test {} completed: {}", i + 1, result.stdout.trim());
            }
            Ok(Err(e)) => {
                eprintln!("Test {} failed: {}", i + 1, e);
            }
            Err(e) => {
                eprintln!("Test {} panicked: {}", i + 1, e);
            }
        }
    }

    Ok(())
}
```

## CLI Integration

### Example 16: CLI Tool with gVisor

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Image to use
    #[arg(short, long, default_value = "alpine:latest")]
    image: String,

    /// Command to run
    command: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.command.is_empty() {
        eprintln!("Error: No command specified");
        std::process::exit(1);
    }

    println!("Using image: {}", cli.image);

    let backend = GvisorBackend::new(&cli.image).await?;

    let cmd = Cmd::new(&cli.command[0])
        .args(&cli.command[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let result = backend.run_cmd(cmd)?;

    print!("{}", result.stdout);
    eprint!("{}", result.stderr);

    std::process::exit(result.exit_code);
}
```

**Usage:**
```bash
cargo run -- --image alpine:latest echo "Hello World"
cargo run -- --image ubuntu:22.04 ls -la /etc
```

## Comparison Examples

### Example 17: Docker vs gVisor

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cmd_spec = Cmd::new("echo").arg("test");

    // Using testcontainers (Docker)
    println!("=== Using Docker ===");
    let start_docker = std::time::Instant::now();
    let docker_backend = TestcontainerBackend::new("alpine:latest")?;
    let docker_result = docker_backend.run_cmd(cmd_spec.clone())?;
    let docker_duration = start_docker.elapsed();
    println!("Duration: {:?}", docker_duration);
    println!("Output: {}", docker_result.stdout);

    // Using gVisor (OCI direct)
    println!("\n=== Using gVisor ===");
    let start_gvisor = std::time::Instant::now();
    let gvisor_backend = GvisorBackend::new("alpine:latest").await?;
    let gvisor_result = gvisor_backend.run_cmd(cmd_spec.clone())?;
    let gvisor_duration = start_gvisor.elapsed();
    println!("Duration: {:?}", gvisor_duration);
    println!("Output: {}", gvisor_result.stdout);

    println!("\n=== Comparison ===");
    println!("Docker: {:?}", docker_duration);
    println!("gVisor: {:?}", gvisor_duration);
    println!("Speedup: {:.2}x", docker_duration.as_secs_f64() / gvisor_duration.as_secs_f64());

    Ok(())
}
```

## Best Practices

### 1. Cache Warming

Pre-load images during CI setup:
```bash
# In CI pipeline
cargo run --example warm_cache alpine:latest ubuntu:22.04
```

### 2. Error Context

Always add context to errors:
```rust
let result = backend.run_cmd(cmd)
    .map_err(|e| e.with_context("Running integration test"))?;
```

### 3. Timeouts

Set appropriate timeouts:
```rust
// Short commands
let backend = GvisorBackend::new("alpine:latest")
    .await?
    .with_timeout(Duration::from_secs(5));

// Database initialization
let backend = GvisorBackend::new("postgres:15")
    .await?
    .with_timeout(Duration::from_secs(60));
```

### 4. Resource Cleanup

Use RAII patterns:
```rust
{
    let backend = GvisorBackend::new("alpine:latest").await?;
    let result = backend.run_cmd(cmd)?;
    // Backend drops here, cleanup automatic
}
```

### 5. Image Naming

Use specific tags for reproducibility:
```rust
// Good
GvisorBackend::new("alpine:3.18")
GvisorBackend::new("postgres:15.2")

// Avoid (non-deterministic)
GvisorBackend::new("alpine:latest")
GvisorBackend::new("postgres")
```
