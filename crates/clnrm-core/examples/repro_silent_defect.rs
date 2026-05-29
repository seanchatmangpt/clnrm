use clnrm_core::backend::oci::cache::ImageCache;
use clnrm_core::backend::oci::layer_manager::LayerManager;
use clnrm_core::backend::oci::{OciImage, OciLayer, OciImageConfig, OciContainerConfig, OciRootfs};
use std::fs;
use tar::Builder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup
    let cache = ImageCache::new(1).map_err(|e| anyhow::anyhow!("{}", e))?;
    let manager = LayerManager::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // 2. Create a dummy image with one plain tar layer
    let layer_content = "original content\n";
    let mut tar_builder = Builder::new(Vec::new());
    
    let mut header = tar::Header::new_gnu();
    header.set_size(layer_content.len() as u64);
    header.set_path("payload.txt")?;
    header.set_mode(0o644);
    header.set_cksum();
    tar_builder.append(&header, layer_content.as_bytes())?;
    
    let tar_data = tar_builder.into_inner()?;
    let digest = "sha256:11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff";
    
    let layer = OciLayer {
        digest: digest.to_string(),
        media_type: "application/vnd.oci.image.layer.v1.tar".to_string(),
        size: tar_data.len() as u64,
        data: tar_data,
    };

    let image_ref = "test-image:latest";
    let config = OciImageConfig {
        architecture: "amd64".to_string(),
        os: "linux".to_string(),
        config: OciContainerConfig {
            user: None,
            exposed_ports: None,
            env: None,
            cmd: None,
            volumes: None,
            working_dir: None,
            entrypoint: None,
            labels: None,
        },
        rootfs: OciRootfs {
            typ: "layers".to_string(),
            diff_ids: vec![digest.to_string()],
        },
        history: None,
    };

    let config_bytes = serde_json::to_vec(&config)?;

    let image = OciImage {
        manifest: Default::default(),
        config,
        layers: vec![layer.clone()],
        config_bytes,
    };

    // 3. Store in cache
    println!("Storing image in blob cache...");
    cache.store(image_ref, &image).await.map_err(|e| anyhow::anyhow!("{}", e))?;

    // 4. Locate the blob on disk
    let blob_path = dirs::cache_dir().unwrap()
        .join("clnrm").join("oci").join("layers")
        .join(digest.replace(':', "_"));
    
    println!("Blob path: {}", blob_path.display());
    if !blob_path.exists() {
        return Err(anyhow::anyhow!("Blob not found at {}", blob_path.display()));
    }

    // 5. Corrupt the blob (change "original" to "CORRUPTED")
    let mut data = fs::read(&blob_path)?;
    let original_str = "original";
    let corrupted_str = "CORRUPTE"; // same length to avoid breaking tar structure too much
    
    let data_str = String::from_utf8_lossy(&data);
    if let Some(pos) = data_str.find(original_str) {
        println!("Found 'original' at pos {}, corrupting...", pos);
        for (i, b) in corrupted_str.as_bytes().iter().enumerate() {
            data[pos + i] = *b;
        }
        fs::write(&blob_path, data)?;
        println!("Blob corrupted.");
    } else {
        return Err(anyhow::anyhow!("Could not find 'original' in blob data"));
    }

    // 6. Retrieve from cache
    println!("Retrieving image from cache...");
    let cached_image = cache.get(image_ref).await.map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Image not found in cache after storage"))?;
    
    let mut retrieved_layer = cached_image.layers[0].clone();
    // Workaround: ImageCache hardcodes media_type to gzip, so we fix it back to tar
    retrieved_layer.media_type = "application/vnd.oci.image.layer.v1.tar".to_string();
    println!("Retrieved layer digest: {}", retrieved_layer.digest);
    
    // 7. Extract to a fresh directory (bypassing LayerManager's directory cache)
    println!("Calling LayerManager::extract_layer_to_cache...");
    let result_dir = manager.extract_layer_to_cache(&retrieved_layer).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    let payload_path = result_dir.join("payload.txt");
    let extracted_content = fs::read_to_string(&payload_path)?;
    println!("Extracted content: {}", extracted_content.trim());

    if extracted_content.contains("CORRUPTE") {
        println!("\n[VULNERABILITY CONFIRMED] Silent Defect Propagation!");
        println!("The system extracted a corrupted blob without verifying its SHA256 digest.");
        println!("The digest was {} but the data actually contained 'CORRUPTE'.", retrieved_layer.digest);
    } else {
        println!("\n[INFO] Corruption was not found in extracted content. content='{}'", extracted_content);
    }

    Ok(())
}
