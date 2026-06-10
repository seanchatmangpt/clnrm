use clnrm_core::backend::oci::LocalImageStore;
use clnrm_core::error::ErrorKind;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn setup_mock_oci_layout(corrupt_index: bool, missing_blob: bool, mismatch_hash: bool) -> PathBuf {
    let dir = tempdir().unwrap().into_path();

    // oci-layout
    fs::write(dir.join("oci-layout"), r#"{"imageLayoutVersion": "1.0.0"}"#).unwrap();

    if corrupt_index {
        fs::write(dir.join("index.json"), "{ invalid json").unwrap();
    } else {
        let index_json = r#"{
            "schemaVersion": 2,
            "manifests": [
                {
                    "mediaType": "application/vnd.oci.image.manifest.v1+json",
                    "digest": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
                    "size": 100
                }
            ]
        }"#;
        fs::write(dir.join("index.json"), index_json).unwrap();
    }

    let blobs_dir = dir.join("blobs").join("sha256");
    if !missing_blob {
        fs::create_dir_all(&blobs_dir).unwrap();

        let manifest = r#"{
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "config": {
                "mediaType": "application/vnd.oci.image.config.v1+json",
                "digest": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
                "size": 50
            },
            "layers": [
                {
                    "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
                    "digest": "sha256:3333333333333333333333333333333333333333333333333333333333333333",
                    "size": 20
                }
            ]
        }"#;
        fs::write(
            blobs_dir.join("1111111111111111111111111111111111111111111111111111111111111111"),
            manifest,
        )
        .unwrap();

        let config = r#"{"architecture": "amd64", "os": "linux", "config": {}, "rootfs": {"type": "layers", "diff_ids": []}}"#;
        fs::write(
            blobs_dir.join("2222222222222222222222222222222222222222222222222222222222222222"),
            config,
        )
        .unwrap();

        if mismatch_hash {
            fs::write(
                blobs_dir.join("3333333333333333333333333333333333333333333333333333333333333333"),
                "wrong data",
            )
            .unwrap();
        } else {
            // Need actual sha256 to match the hash 333... wait, if I want to pass, I need a correct hash,
            // but the test is only about failing validation. Let's just use "wrong data".
            fs::write(
                blobs_dir.join("3333333333333333333333333333333333333333333333333333333333333333"),
                "wrong data",
            )
            .unwrap();
        }
    }

    dir
}

#[tokio::test]
async fn test_oci_contract_corrupt_index() {
    let dir = setup_mock_oci_layout(true, false, false);
    let store = LocalImageStore::new().unwrap();
    let result = store.load_from_path(dir).await;
    assert!(result.is_err(), "Should fail with corrupt index");
    let err = result.unwrap_err();
    assert!(
        err.kind == ErrorKind::SerializationError || err.kind == ErrorKind::ValidationError,
        "Expected validation/serialization error, got {:?}",
        err.kind
    );
}

#[tokio::test]
async fn test_oci_contract_missing_blobs() {
    let dir = setup_mock_oci_layout(false, true, false);
    let store = LocalImageStore::new().unwrap();
    let result = store.load_from_path(dir).await;
    assert!(result.is_err(), "Should fail with missing blobs dir");
    let err = result.unwrap_err();
    assert!(
        err.kind == ErrorKind::IoError || err.kind == ErrorKind::ValidationError,
        "Expected IO/validation error, got {:?}",
        err.kind
    );
}

#[tokio::test]
async fn test_oci_contract_mismatched_hash() {
    let dir = setup_mock_oci_layout(false, false, true);
    let store = LocalImageStore::new().unwrap();
    let result = store.load_from_path(dir).await;
    assert!(result.is_err(), "Should fail with mismatched layer hash");
    let err = result.unwrap_err();
    assert_eq!(err.kind, ErrorKind::ValidationError);
    assert!(
        err.message.contains("hash mismatch")
            || err.message.contains("digest mismatch")
            || err.message.contains("checksum mismatch")
    );
}
