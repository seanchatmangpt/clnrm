use crate::backend::oci::{ImageSource, OciBundleBuilder, OciImageLoader, RunscExecutor};
use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use tracing::info;

#[allow(dead_code)]
const SURREALDB_PORT: u16 = 8000;

#[derive(Debug)]
pub struct SurrealDbPlugin {
    pub name: String,
    pub image: String,
}

impl Default for SurrealDbPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl SurrealDbPlugin {
    pub fn new() -> Self {
        Self {
            name: "surrealdb".to_string(),
            image: "surrealdb/surrealdb:latest".to_string(),
        }
    }

    pub fn with_credentials(_user: &str, _pass: &str) -> Self {
        Self::new()
    }
    pub fn with_strict(self, _strict: bool) -> Self {
        self
    }
    pub fn with_name(self, name: impl Into<String>) -> Self {
        let mut s = self;
        s.name = name.into();
        s
    }
}

impl ServicePlugin for SurrealDbPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<crate::cleanroom::ServiceHandle> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CleanroomError::internal_error(e.to_string()))?;
        rt.block_on(async {
            info!("Starting SurrealDB service '{}' via gVisor", self.name);

            // 1. Initialize OCI tools
            let loader = OciImageLoader::new()?;
            let source = ImageSource::Registry {
                registry: "registry-1.docker.io".to_string(),
                repository: "surrealdb/surrealdb".to_string(),
                tag: "latest".to_string(),
            };

            // 2. Load image
            let image = loader.load_image(source).await?;

            // 3. Create OCI bundle
            let builder = OciBundleBuilder::new()?;
            // Create bundle with default SurrealDB command
            let bundle = builder.create_bundle(&image, None, None).await?;

            // 4. Execute with runsc
            let executor = RunscExecutor::new()?;
            let container_id = format!("surrealdb-{}", uuid::Uuid::new_v4());

            executor.create_container(&container_id, &bundle).await?;
            executor.start_container(&container_id).await?;

            let mut handle = ServiceHandle::new(&self.name);
            handle
                .metadata
                .insert("container_id".to_string(), container_id.clone());
            handle.metadata.insert(
                "bundle_path".to_string(),
                bundle.path.to_string_lossy().into_owned(),
            );
            handle
                .metadata
                .insert("port".to_string(), "8000".to_string());

            info!("SurrealDB service started: {}", container_id);
            Ok(handle)
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CleanroomError::internal_error(e.to_string()))?;
        rt.block_on(async {
            if let Some(container_id) = handle.metadata.get("container_id") {
                let executor = RunscExecutor::new()?;
                info!("Stopping SurrealDB container: {}", container_id);
                let _ = executor.kill_container(container_id).await;
                let _ = executor.delete_container(container_id).await;
            }

            if let Some(bundle_path_str) = handle.metadata.get("bundle_path") {
                let bundle_path = std::path::PathBuf::from(bundle_path_str);
                if bundle_path.exists() {
                    let _ = std::fs::remove_dir_all(bundle_path);
                }
            }
            Ok(())
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        let port = handle
            .metadata
            .get("port")
            .map(|s| s.as_str())
            .unwrap_or("8000");
        let addr = format!("127.0.0.1:{}", port);

        match std::net::TcpStream::connect_timeout(
            &addr
                .parse()
                .unwrap_or(std::net::SocketAddr::from(([127, 0, 0, 1], 8000))),
            std::time::Duration::from_millis(50),
        ) {
            Ok(_) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }
}
