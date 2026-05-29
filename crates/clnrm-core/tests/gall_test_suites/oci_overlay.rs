//! Gall Test Suite for OCI Overlay Filesystem
//!
//! Exposes the performance gap where the framework currently extracts
//! full root filesystems instead of using lightweight overlay mounts.

use clnrm_core::backend::oci::LayerManager;

#[tokio::test]
async fn gall_gap_test_oci_overlay_fs_mount() {
    // Arrange
    let _manager = LayerManager::new().unwrap();
    
    // Act & Assert
    // GALL GAP: We currently use `extract_rootfs` which does full tar decompression.
    // The primitive required for millisecond container scaling is an overlayfs mount.
    // We expect a method like `mount_overlayfs(&self, layers, target)` to exist and be used.
    
    panic!("Gall Gap: OCI Overlay FS missing. LayerManager performs slow full extraction instead of rapid overlayfs mounting. `temp_dir` and `cache_dir` fields are dead code.");
}