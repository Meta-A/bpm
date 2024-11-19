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
    use core::services::{
        db::packages_repository::PackagesRepository, package_managers::PackageManagersService,
        packages::PackagesService,
    };

    init_logger(log::LevelFilter::Info);
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    info!("BPM v{}", VERSION);

    let config_path = home_dir().unwrap();

    let mut config_manager = init_config(&config_path);

    let db_client = Arc::new(DbClient::from(&config_manager.get_db_path()));

    // Repositories
    let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
    let packages_repository = Arc::new(PackagesRepository::from(&db_client));

    // Services
    let package_managers_service = Arc::new(PackageManagersService::new());

    package_managers_service.init_package_managers().await;

    let packages_service = Arc::new(PackagesService::from(&packages_repository));
    let blockchains_service =
        Arc::new(BlockchainsService::new(&blockchains_repository, &packages_service).await);

    blockchains_service.init_blockchains().await;

    commands::bootstrap(
        &mut config_manager,
        &blockchains_service,
        &packages_service,
        &package_managers_service,
    )
    .await?;

    Ok(())
}
