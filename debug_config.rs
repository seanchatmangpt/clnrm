use clnrm_core::config::load_cleanroom_config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_cleanroom_config()?;
    println!("Default image: {}", config.containers.default_image);
    Ok(())
}
