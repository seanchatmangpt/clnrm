//! OCI image management

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Pull OCI image for gVisor
#[verb("pull")]
pub fn pull(image: String) -> Result<String> {
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(clnrm_core::cli::commands::pull_oci_image(&image))
    });

    match result {
        Ok(oci_image) => Ok(format!(
            "Successfully pulled image: {}. Digest: {}",
            image, oci_image.manifest.config.digest
        )),
        Err(e) => Ok(format!("Failed to pull image: {}. Error: {}", image, e)),
    }
}
