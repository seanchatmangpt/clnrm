# clnrm pull - Docker Image Pre-Pulling

## Overview

The `clnrm pull` command pre-pulls Docker images from test configurations to avoid delays during test execution. This is especially useful in CI/CD environments where image pull times can significantly impact build times.

## Usage

```bash
# Pull all images from test files in current directory
clnrm pull

# Pull images from specific test files
clnrm pull tests/integration.clnrm.toml tests/e2e.clnrm.toml

# Pull images from a directory (recursive)
clnrm pull tests/

# Pull images in parallel (default: sequential)
clnrm pull --parallel

# Pull images with custom parallelism
clnrm pull --parallel --jobs 8
```

## Command Options

```
clnrm pull [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  Test files or directories to scan for images (default: all test files)

Options:
  -p, --parallel     Pull images in parallel
  -j, --jobs <JOBS>  Maximum parallel pulls [default: 4]
  -h, --help         Print help
```

## How It Works

1. **Discovery**: Scans test files for service configurations
2. **Extraction**: Extracts unique Docker images from `services` or `service` sections
3. **Pulling**: Pulls images using Docker CLI (`docker pull`)
4. **Progress**: Shows progress for each image pull

## Test File Format

The command supports both old and new test file formats:

### v0.6.0+ Format (with service section)

```toml
[test.metadata]
name = "integration_test"
description = "Integration test"

[service.database]
type = "container"
plugin = "testcontainers"
image = "postgres:15-alpine"

[service.redis]
type = "container"
plugin = "testcontainers"
image = "redis:7-alpine"
```

### v0.4.x Format (with services section)

```toml
[test.metadata]
name = "integration_test"
description = "Integration test"

[services]
database = { type = "container", plugin = "testcontainers", image = "postgres:15-alpine" }
redis = { type = "container", plugin = "testcontainers", image = "redis:7-alpine" }
```

## Features

### Parallel Pulling

Use `--parallel` and `--jobs` to pull multiple images concurrently:

```bash
# Pull up to 8 images in parallel
clnrm pull --parallel --jobs 8 tests/
```

This is useful for large test suites with many different images.

### Recursive Discovery

The command automatically discovers test files recursively:

```bash
# Discovers all .clnrm.toml files in tests/ and subdirectories
clnrm pull tests/
```

### Deduplication

Images are automatically deduplicated - if multiple test files use the same image, it will only be pulled once.

### Error Handling

- Gracefully handles missing Docker daemon
- Reports failed image pulls with detailed error messages
- Continues pulling remaining images after failures (in parallel mode)

## Examples

### CI/CD Pipeline Integration

```yaml
# GitHub Actions example
steps:
  - name: Pre-pull test images
    run: clnrm pull --parallel --jobs 4 tests/

  - name: Run tests
    run: clnrm run tests/
```

### Development Workflow

```bash
# Pull images before running tests
clnrm pull tests/integration/

# Run tests (images already cached)
clnrm run tests/integration/
```

## Performance

Pulling images in advance can significantly reduce test execution time:

- **Sequential pulling**: ~30-60s per image
- **Parallel pulling (4 jobs)**: ~10-20s per batch of 4 images
- **Test execution**: Uses cached images (instant startup)

## Implementation Details

### Architecture

- **Location**: `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/pull.rs`
- **Entry Point**: `pull_images()` function
- **CLI Handler**: `/Users/sac/clnrm/crates/clnrm-core/src/cli/mod.rs`

### Key Functions

1. **`pull_images()`**: Main entry point, orchestrates discovery and pulling
2. **`discover_test_files_from_paths()`**: Discovers test files from paths
3. **`extract_images_from_test_files()`**: Parses TOML and extracts image names
4. **`pull_images_sequential()`**: Pulls images one at a time
5. **`pull_images_parallel()`**: Pulls images concurrently with semaphore control
6. **`pull_single_image()`**: Executes `docker pull` for a single image

### Error Handling

- Uses `Result<T, CleanroomError>` for all operations
- No `.unwrap()` or `.expect()` in production code
- Proper error context with file paths and error messages
- Clean error propagation with `?` operator

### Concurrency

- Uses `tokio::sync::Semaphore` for parallel job control
- Spawns tasks with `tokio::spawn`
- Properly awaits all tasks with join error handling

## Troubleshooting

### "Docker daemon not running"

```
Error: Failed to pull image alpine:latest: Cannot connect to the Docker daemon
```

**Solution**: Start Docker daemon:
```bash
# macOS
open -a Docker

# Linux
sudo systemctl start docker
```

### "Permission denied"

```
Error: Got permission denied while trying to connect to the Docker daemon socket
```

**Solution**: Add user to docker group:
```bash
sudo usermod -aG docker $USER
newgrp docker
```

### "Failed to parse test file"

```
Error: Failed to parse test file: missing field `metadata`
```

**Solution**: Ensure test file has proper format with `[test.metadata]` section.

## Future Enhancements

Potential future features:

1. **Image cache statistics**: Show which images are already cached
2. **Selective pulling**: `--only-missing` flag to skip cached images
3. **Progress bars**: Better visual feedback for large pulls
4. **Registry authentication**: Support for private registries
5. **Image verification**: Verify image digests after pulling
6. **Image list output**: `--list` flag to show images without pulling

## See Also

- [CLI Guide](CLI_GUIDE.md)
- [Test Configuration](TOML_REFERENCE.md)
- [Service Plugins](PLUGINS.md)
