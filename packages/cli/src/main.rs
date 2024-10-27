mod commands;

use core::config::init_config;
use core::logging::init_logger;
use home::home_dir;
use log::info;

/**
 * Main CLI entry point
 */
#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger(log::LevelFilter::Debug);
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    info!("BBPM v{}", VERSION);

    let config_path = home_dir().unwrap();

    let mut config_manager = init_config(&config_path);

    commands::bootstrap(&mut config_manager).await?;

    Ok(())
}
