mod commands;

use core::config::init_config;
use core::logging::init_logger;
use home::home_dir;
use log::info;

use std::sync::Arc;

use core::{
    db::client::DbClient,
    services::{
        blockchains::BlockchainsService, db::blockchains_repository::BlockchainsRepository,
    },
};

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

    let db_client = Arc::new(DbClient::from(&config_manager.get_db_path()));

    let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));

    let blockchains_service = Arc::new(BlockchainsService::from(&blockchains_repository));

    blockchains_service.init_blockchains().await;

    commands::bootstrap(&mut config_manager, &blockchains_service).await?;

    Ok(())
}
