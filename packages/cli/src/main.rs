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
async fn main() {
    init_logger(log::LevelFilter::Info);

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    info!("BBPM v{}", VERSION);

    let config_path = home_dir().unwrap();

    init_config(&config_path);

    commands::bootstrap().await;
}
