mod commands;

use bpm_core::config::init_config;
use bpm_core::logging::init_logger;
use home::home_dir;
use log::info;

use std::sync::Arc;

use bpm_core::{
    blockchains::get_available_clients,
    db::client::DbClient,
    package_managers::init_package_managers,
    services::{
        blockchains::BlockchainsService, db::blockchains_repository::BlockchainsRepository,
        db::packages_repository::PackagesRepository, package_managers::PackageManagersService,
        packages::PackagesService,
    },
};

/**
 * Main CLI entry point
 */
#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger(log::LevelFilter::Info);
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    info!("BPM v{}", VERSION);

    let config_path = home_dir().unwrap();

    let mut config_manager = init_config(&config_path);

    let db_client = Arc::new(DbClient::from(&config_manager.get_db_path()));

    // Blockchains clients
    let available_blockchains = get_available_clients();

    // Package managers
    let available_package_managers = init_package_managers().await;

    // Repositories
    let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
    let packages_repository = Arc::new(PackagesRepository::from(&db_client));

    // Services
    let package_managers_service =
        Arc::new(PackageManagersService::new(&available_package_managers));

    let packages_service = Arc::new(PackagesService::from(&packages_repository));

    let blockchains_service = Arc::new(
        BlockchainsService::new(
            &available_blockchains,
            &blockchains_repository,
            &packages_service,
        )
        .await,
    );

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
