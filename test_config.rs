use clnrm_core::config::load_cleanroom_config;

#[tokio::test]
async fn test_config_loading() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_cleanroom_config()?;
    println!("Default image: {}", config.containers.default_image);
    Ok(())
}
