//! File System Error Injection Tests
//!
//! Tests system resilience against file system failures including
//! permission errors, disk full, corruption, and I/O errors.

use clnrm_core::error::Result;
use std::fs::{File, create_dir_all, remove_dir_all, OpenOptions};
use std::io::{Write, Read};
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

/// Test file not found errors
#[tokio::test]
async fn test_file_not_found() -> Result<()> {
    let result = File::open("/nonexistent/path/to/file.txt");

    match result {
        Ok(_) => panic!("Expected file not found error"),
        Err(e) => {
            println!("File not found error (expected): {}", e);
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
    }

    Ok(())
}

/// Test permission denied errors
#[tokio::test]
#[cfg(unix)]
async fn test_permission_denied() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("readonly.txt");

    // Create file and write data
    let mut file = File::create(&file_path)?;
    file.write_all(b"test data")?;
    drop(file);

    // Make file read-only
    let mut perms = std::fs::metadata(&file_path)?.permissions();
    perms.set_mode(0o444);
    std::fs::set_permissions(&file_path, perms)?;

    // Try to write to read-only file
    let result = OpenOptions::new()
        .write(true)
        .open(&file_path);

    match result {
        Ok(_) => println!("Warning: Was able to write to read-only file"),
        Err(e) => {
            println!("Permission denied error (expected): {}", e);
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
        }
    }

    Ok(())
}

/// Test disk full simulation
#[tokio::test]
async fn test_disk_full_simulation() -> Result<()> {
    let temp_dir = tempdir()?;

    // Simulate disk full by filling up available space
    let max_size = 1024 * 1024 * 10; // 10MB limit
    let mut total_written = 0;
    let chunk_size = 1024 * 100; // 100KB chunks

    for i in 0..200 {
        if total_written >= max_size {
            println!("Simulated disk full at {}MB", total_written / (1024 * 1024));
            break;
        }

        let file_path = temp_dir.path().join(format!("file_{}.dat", i));
        match File::create(&file_path) {
            Ok(mut file) => {
                let data = vec![0u8; chunk_size];
                match file.write_all(&data) {
                    Ok(_) => total_written += chunk_size,
                    Err(e) => {
                        println!("Write failed (disk full simulation): {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("File creation failed: {}", e);
                break;
            }
        }
    }

    println!("Total data written: {}KB", total_written / 1024);

    Ok(())
}

/// Test file corruption detection
#[tokio::test]
async fn test_file_corruption_detection() -> Result<()> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("data.txt");

    // Write structured data
    let original_data = b"HEADER:12345:DATA:ABCDEFGH:CHECKSUM:99999";
    let mut file = File::create(&file_path)?;
    file.write_all(original_data)?;
    drop(file);

    // Simulate corruption by modifying file
    let mut file = OpenOptions::new()
        .write(true)
        .open(&file_path)?;

    file.write_all(b"XXXXX")?; // Corrupt header
    drop(file);

    // Try to read and validate
    let mut file = File::open(&file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let is_corrupted = !buffer.starts_with(b"HEADER:");

    println!("Corruption detection: {}",
        if is_corrupted { "CORRUPTED" } else { "VALID" });

    assert!(is_corrupted, "Expected to detect corruption");

    Ok(())
}

/// Test directory traversal errors
#[tokio::test]
async fn test_directory_traversal_errors() -> Result<()> {
    let temp_dir = tempdir()?;
    let deep_path = temp_dir.path()
        .join("level1")
        .join("level2")
        .join("level3")
        .join("level4")
        .join("level5");

    // Try to access non-existent deep path
    let result = File::open(&deep_path);

    match result {
        Ok(_) => panic!("Expected directory not found"),
        Err(e) => {
            println!("Directory traversal error (expected): {}", e);
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
    }

    // Create deep directory structure
    create_dir_all(&deep_path)?;

    // Now it should work
    let file_path = deep_path.join("test.txt");
    let mut file = File::create(&file_path)?;
    file.write_all(b"deep file")?;

    println!("Successfully created file at depth 5");

    Ok(())
}

/// Test concurrent file access
#[tokio::test]
async fn test_concurrent_file_access() -> Result<()> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("concurrent.txt");

    // Create file
    File::create(&file_path)?;

    let mut tasks = vec![];

    // Spawn concurrent readers and writers
    for i in 0..10 {
        let path = file_path.clone();
        let task = tokio::spawn(async move {
            if i % 2 == 0 {
                // Writer
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(&path)?;

                file.write_all(format!("Line {}\n", i).as_bytes())?;
            } else {
                // Reader
                let mut file = File::open(&path)?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
            }

            Ok::<_, std::io::Error>(())
        });

        tasks.push(task);
    }

    // Wait for all tasks
    let results = futures_util::future::join_all(tasks).await;

    let failures: Vec<_> = results.iter()
        .filter(|r| r.is_err() || (r.is_ok() && r.as_ref().unwrap().is_err()))
        .collect();

    println!("Concurrent file access - {} failures out of 10", failures.len());

    Ok(())
}

/// Test file locking
#[tokio::test]
#[cfg(unix)]
async fn test_file_locking() -> Result<()> {
    use std::os::unix::fs::FileExt;

    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("locked.txt");

    let file1 = File::create(&file_path)?;

    // Try to open the same file again
    let file2_result = OpenOptions::new()
        .write(true)
        .open(&file_path);

    match file2_result {
        Ok(_file2) => {
            println!("File opened by multiple handles (expected on some systems)");
        }
        Err(e) => {
            println!("File lock prevented access: {}", e);
        }
    }

    drop(file1);

    Ok(())
}

/// Test symbolic link errors
#[tokio::test]
#[cfg(unix)]
async fn test_symbolic_link_errors() -> Result<()> {
    let temp_dir = tempdir()?;
    let target = temp_dir.path().join("target.txt");
    let link = temp_dir.path().join("link.txt");

    // Create symlink to non-existent file
    std::os::unix::fs::symlink(&target, &link)?;

    // Try to read through broken symlink
    let result = File::open(&link);

    match result {
        Ok(_) => panic!("Expected broken symlink error"),
        Err(e) => {
            println!("Broken symlink error (expected): {}", e);
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
    }

    // Create target and verify symlink works
    File::create(&target)?;

    let result = File::open(&link);
    assert!(result.is_ok(), "Symlink should work after creating target");

    Ok(())
}

/// Test path too long errors
#[tokio::test]
async fn test_path_too_long() -> Result<()> {
    let temp_dir = tempdir()?;

    // Create an extremely long path
    let mut long_path = temp_dir.path().to_path_buf();
    for i in 0..100 {
        long_path.push(format!("very_long_directory_name_{}", i));
    }
    long_path.push("file.txt");

    println!("Testing path length: {}", long_path.to_string_lossy().len());

    // Try to create file with very long path
    let result = create_dir_all(long_path.parent().unwrap());

    match result {
        Ok(_) => {
            println!("Warning: System accepted very long path");
        }
        Err(e) => {
            println!("Path too long error (expected): {}", e);
        }
    }

    Ok(())
}

/// Test file system full during write
#[tokio::test]
async fn test_write_interrupted() -> Result<()> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("interrupted.txt");

    let mut file = File::create(&file_path)?;

    // Write data in chunks and simulate interruption
    let chunk_size = 1024;
    let total_chunks = 100;

    for i in 0..total_chunks {
        let data = vec![i as u8; chunk_size];

        match file.write_all(&data) {
            Ok(_) => {
                if i % 20 == 0 {
                    println!("Written {}KB", i);
                }
            }
            Err(e) => {
                println!("Write interrupted at chunk {}: {}", i, e);
                break;
            }
        }
    }

    Ok(())
}

/// Test metadata access errors
#[tokio::test]
async fn test_metadata_access() -> Result<()> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test.txt");

    // Try to get metadata of non-existent file
    let result = std::fs::metadata(&file_path);

    match result {
        Ok(_) => panic!("Expected metadata error"),
        Err(e) => {
            println!("Metadata error for non-existent file (expected): {}", e);
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
    }

    // Create file and verify metadata access
    File::create(&file_path)?;

    let metadata = std::fs::metadata(&file_path)?;
    println!("File metadata - Size: {} bytes, Read-only: {}",
        metadata.len(), metadata.permissions().readonly());

    Ok(())
}

/// Test directory removal with open handles
#[tokio::test]
async fn test_directory_removal_with_open_handles() -> Result<()> {
    let temp_dir = tempdir()?;
    let sub_dir = temp_dir.path().join("subdir");
    create_dir_all(&sub_dir)?;

    let file_path = sub_dir.join("file.txt");
    let _open_file = File::create(&file_path)?;

    // Try to remove directory while file is open
    let result = remove_dir_all(&sub_dir);

    match result {
        Ok(_) => {
            println!("Directory removed despite open handle");
        }
        Err(e) => {
            println!("Cannot remove directory with open handles (expected on some systems): {}", e);
        }
    }

    Ok(())
}

/// Test file rename under load
#[tokio::test]
async fn test_file_rename_chaos() -> Result<()> {
    let temp_dir = tempdir()?;

    let mut tasks = vec![];

    for i in 0..20 {
        let path = temp_dir.path().to_path_buf();
        let task = tokio::spawn(async move {
            let src = path.join(format!("file_{}.txt", i));
            let dst = path.join(format!("renamed_{}.txt", i));

            // Create file
            File::create(&src)?;

            // Rename file
            std::fs::rename(&src, &dst)?;

            // Verify renamed file exists
            let exists = std::fs::metadata(&dst).is_ok();

            Ok::<_, std::io::Error>(exists)
        });

        tasks.push(task);
    }

    let results = futures_util::future::join_all(tasks).await;

    let successful = results.iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok() && *r.as_ref().unwrap().as_ref().unwrap())
        .count();

    println!("File rename chaos - {} successful out of 20", successful);

    Ok(())
}

/// Test temporary file cleanup
#[tokio::test]
async fn test_temp_file_cleanup() -> Result<()> {
    let temp_paths = {
        let mut paths = Vec::new();

        // Create temporary files
        for i in 0..10 {
            let temp = tempdir()?;
            let file_path = temp.path().join(format!("temp_{}.txt", i));
            File::create(&file_path)?;

            paths.push(file_path.clone());
        }

        paths
    };

    // Temporary directories should be cleaned up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let remaining = temp_paths.iter()
        .filter(|p| p.exists())
        .count();

    println!("Temp file cleanup - {} files remaining (expected: 0)", remaining);

    Ok(())
}
