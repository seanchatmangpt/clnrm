use clnrm_core::backend::oci::layer_manager::LayerManager;
use clnrm_core::backend::oci::OciLayer;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use tar::Builder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup LayerManager
    let manager = LayerManager::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    let target_dir = tempdir()?;
    let rootfs_mount = target_dir.path().join("mountpoint");
    fs::create_dir_all(&rootfs_mount)?;

    // 2. Create a dummy layer
    let layer_content = "original content\n";
    let mut tar_builder = Builder::new(Vec::new());
    
    let mut header = tar::Header::new_gnu();
    header.set_size(layer_content.len() as u64);
    header.set_path("hello.txt")?;
    header.set_mode(0o644);
    header.set_cksum();
    tar_builder.append(&header, layer_content.as_bytes())?;
    
    let tar_data = tar_builder.into_inner()?;
    let digest = "sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    
    let layer = OciLayer {
        digest: digest.to_string(),
        media_type: "application/vnd.oci.image.layer.v1.tar".to_string(),
        size: tar_data.len() as u64,
        data: tar_data,
    };

    println!("Extracting layer to cache...");
    let layer_dir = manager.extract_layer_to_cache(&layer).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let hello_path_in_cache = layer_dir.join("hello.txt");
    let content = fs::read_to_string(&hello_path_in_cache)?;
    println!("Content in cache: {}", content.trim());
    assert_eq!(content, layer_content);

    // 3. Attempt Mount (Linux only)
    let rootfs_res = manager.mount_overlayfs(&[layer.clone()], &rootfs_mount).await;
    
    match rootfs_res {
        Ok(rootfs) => {
            let hello_path = rootfs.join("hello.txt");
            println!("Mounted OverlayFS at {}", rootfs.display());
            
            // 4. Tamper
            println!("Tampering with cache at: {}", hello_path_in_cache.display());
            let tampered_content = "TAMPERED content\n";
            fs::write(&hello_path_in_cache, tampered_content)?;

            // 5. Read from merged mount
            let content_after = fs::read_to_string(&hello_path)?;
            println!("Content in merged mount after tampering cache: {}", content_after.trim());

            if content_after == tampered_content {
                println!("\n[VULNERABILITY DETECTED] Lack of Tamper Resistance!");
                println!("The framework did not detect that the underlying layer was modified out-of-band.");
            }
            
            let _ = tokio::process::Command::new("umount").arg(&rootfs).output().await;
        },
        Err(e) => {
            println!("\n[INFO] Mount failed (expected on non-Linux): {}", e);
            println!("Demonstrating Cache Pollution instead...");

            // 4. Tamper with cache
            println!("Tampering with cache at: {}", hello_path_in_cache.display());
            let tampered_content = "TAMPERED content\n";
            fs::write(&hello_path_in_cache, tampered_content)?;

            // 5. Ask LayerManager for the same layer again
            println!("Asking LayerManager for the same layer again...");
            let layer_dir_2 = manager.extract_layer_to_cache(&layer).await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            
            let content_2 = fs::read_to_string(layer_dir_2.join("hello.txt"))?;
            println!("Content returned by LayerManager: {}", content_2.trim());

            if content_2 == tampered_content {
                println!("\n[VULNERABILITY DETECTED] Lack of Tamper Resistance (Cache Pollution)!");
                println!("The LayerManager returned a tampered directory from cache without verification.");
                println!("Any container started using this cached layer will be compromised.");
            }
        }
    }

    Ok(())
}
