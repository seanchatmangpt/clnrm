//! Service loading and management for test execution
//!
//! Handles loading services from configuration and registering them with the
//! cleanroom environment.

use crate::cleanroom::CleanroomEnvironment;
use crate::error::{CleanroomError, Result};
use crate::telemetry::spans;
use std::collections::HashMap;
use tracing::{debug, info};

/// Load services from configuration and register them with the environment
pub async fn load_services_from_config(
    env: &CleanroomEnvironment,
    services: &HashMap<String, crate::config::ServiceConfig>,
) -> Result<HashMap<String, crate::cleanroom::ServiceHandle>> {
    let mut service_handles = HashMap::new();

    for (service_name, service_config) in services {
        debug!(
            "Loading service: {} (type: {}, plugin: {})",
            service_name, service_config.plugin, service_config.plugin
        );

        // Create plugin based on service type
        let plugin: Box<dyn crate::cleanroom::ServicePlugin> =
            match service_config.plugin.as_str() {
                "surrealdb" => {
                    use crate::services::surrealdb::SurrealDbPlugin;

                    let username = service_config.username.as_deref().unwrap_or("root");
                    let password = service_config.password.as_deref().unwrap_or("root");
                    let strict = service_config.strict.unwrap_or(false);

                    let plugin = SurrealDbPlugin::with_credentials(username, password)
                        .with_name(service_name)
                        .with_strict(strict);

                    Box::new(plugin)
                }
                "generic_container" => {
                    use crate::services::generic::GenericContainerPlugin;

                    let image = service_config.image.as_deref().ok_or_else(|| {
                        CleanroomError::validation_error(format!(
                            "Service '{}': generic_container requires 'image' field",
                            service_name
                        ))
                    })?;

                    let mut plugin = GenericContainerPlugin::new(service_name, image);

                    if let Some(env_vars) = &service_config.env {
                        for (key, value) in env_vars {
                            plugin = plugin.with_env(key, value);
                        }
                    }

                    if let Some(ports) = &service_config.ports {
                        for port in ports {
                            plugin = plugin.with_port(*port);
                        }
                    }

                    if let Some(volumes) = &service_config.volumes {
                        for volume in volumes {
                            plugin = plugin
                                .with_volume(
                                    &volume.host_path,
                                    &volume.container_path,
                                    volume.read_only.unwrap_or(false),
                                )
                                .map_err(|e| {
                                    CleanroomError::validation_error(format!(
                                        "Service '{}': invalid volume configuration: {}",
                                        service_name, e
                                    ))
                                })?;
                        }
                    }

                    Box::new(plugin)
                }
                _ => {
                    return Err(CleanroomError::validation_error(format!(
                        "Unknown service plugin: {}",
                        service_config.plugin
                    )));
                }
            };

        env.register_service(plugin).await?;
        info!("📦 Registered service plugin: {}", service_name);

        let service_span = spans::service_start_span(service_name, &service_config.plugin);

        let _service_guard = service_span.enter();

        let handle = env.start_service(service_name).await.map_err(|e| {
            CleanroomError::service_error(format!("Failed to start service '{}'", service_name))
                .with_context("Service startup failed")
                .with_source(e.to_string())
        })?;

        info!(
            "✅ Service '{}' started successfully (handle: {})",
            service_name, handle.id
        );

        service_handles.insert(service_name.clone(), handle);
    }

    Ok(service_handles)
}
