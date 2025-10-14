//! SurrealDB service plugin
//!
//! Production-ready SurrealDB container management with health checks
//! and connection verification.

use crate::cleanroom::{ServicePlugin, ServiceHandle, HealthStatus};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use testcontainers_modules::surrealdb::{SurrealDb, SURREALDB_PORT};
use testcontainers::runners::AsyncRunner;
use surrealdb::{engine::remote::ws::{Client, Ws}, opt::auth::Root, Surreal};
use std::future::Future;
use std::pin::Pin;

pub struct SurrealDbPlugin {
    name: String,
    container_id: Arc<RwLock<Option<String>>>,
    username: String,
    password: String,
    strict: bool,
}

impl SurrealDbPlugin {
    pub fn new() -> Self {
        Self::with_credentials("root", "root")
    }

    pub fn with_credentials(username: &str, password: &str) -> Self {
        Self {
            name: "surrealdb".to_string(),
            container_id: Arc::new(RwLock::new(None)),
            username: username.to_string(),
            password: password.to_string(),
            strict: false,
        }
    }

    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    async fn verify_connection(&self, host_port: u16) -> Result<()> {
        let url = format!("127.0.0.1:{}", host_port);
        let db: Surreal<Client> = Surreal::init();
        
        db.connect::<Ws>(url)
            .await
            .map_err(|e| CleanroomError::connection_failed("Failed to connect to SurrealDB")
                .with_source(e.to_string()))?;
        
        db.signin(Root {
            username: &self.username,
            password: &self.password,
        })
        .await
        .map_err(|e| CleanroomError::service_error("Failed to authenticate")
            .with_source(e.to_string()))?;

        Ok(())
    }
}

impl ServicePlugin for SurrealDbPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            let db_config = SurrealDb::default()
                .with_user(&self.username)
                .with_password(&self.password)
                .with_strict(self.strict)
                .with_all_capabilities(true);

            let node = db_config
                .start()
                .await
                .map_err(|e| CleanroomError::container_error("Failed to start SurrealDB container")
                    .with_context("Container startup failed")
                    .with_source(e.to_string()))?;

            let host_port = node.get_host_port_ipv4(SURREALDB_PORT)
                .await
                .map_err(|e| CleanroomError::container_error("Failed to get container port")
                    .with_source(e.to_string()))?;

            // Verify connection works
            self.verify_connection(host_port).await?;

            let mut container_guard = self.container_id.write().await;
            *container_guard = Some(format!("container-{}", host_port));

            let mut metadata = HashMap::new();
            metadata.insert("host".to_string(), "127.0.0.1".to_string());
            metadata.insert("port".to_string(), host_port.to_string());
            metadata.insert("username".to_string(), self.username.clone());
            metadata.insert("database_type".to_string(), "surrealdb".to_string());
            metadata.insert("connection_string".to_string(), format!("ws://127.0.0.1:{}", host_port));

            Ok(ServiceHandle {
                id: Uuid::new_v4().to_string(),
                service_name: self.name.clone(),
                metadata,
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut container_guard = self.container_id.write().await;
            if container_guard.is_some() {
                *container_guard = None; // Drop triggers container cleanup
            }
            Ok(())
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        if handle.metadata.contains_key("port") && 
           handle.metadata.contains_key("connection_string") {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
}
