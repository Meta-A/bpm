use core::blockchains::errors::blockchain_error::BlockchainError;
use core::packages::package::PackageStatus;
use core::services::blockchains::BlockchainsService;
use core::{config::manager::ConfigManager, services::package_managers::PackageManagersService};
use std::sync::Arc;
use std::time::Duration;

use colored::Colorize;

use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
use tokio::sync::mpsc;

/** Install package using its name */
#[derive(Debug, Parser)]
pub struct InstallCommand {
    #[clap(required = true)]
    pub package_name: Option<String>,

    #[clap(required = false)]
    pub package_version: Option<String>,
}

/**
 * Handles package installation request from CLI
 */
impl InstallCommand {
    /**
     * Build progress bar
     */
    fn build_progress_bar(&self) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(60));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                .tick_strings(&[
                    "[    ]", "[=   ]", "[==  ]", "[=== ]", "[====]", "[ ===]", "[  ==]", "[   =]",
                    "[    ]", "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[====]",
                ]),
        );

        pb
    }

    /**
     * Update available packages mutations from blockchain
     */
    async fn update(&self, blockchains_service: &Arc<BlockchainsService>) {
        let (tx_packages_update, mut rx_packages_update) = mpsc::channel(1);

        let task_blockchains_service_ref = Arc::clone(&blockchains_service);
        tokio::spawn(async move {
            let task_res = task_blockchains_service_ref.update(&tx_packages_update);

            match task_res.await {
                Ok(_) => return,
                Err(e) => match e {
                    BlockchainError::NoPackagesData => {
                        info!("No new packages mutations found")
                    }
                    _ => error!("Unhandled error : {}", e),
                },
            }
        });

        let mut packages_count: u128 = 0;

        let pb = self.build_progress_bar();
        pb.set_message("Updating blockchain DB...");

        while let Some(package) = rx_packages_update.recv().await {
            //pb.println(format!(
            //    "New package mutation found => {}:{} ( Maintainer : {}, Status : {}  )",
            //    package.name,
            //    package.version,
            //    hex::encode_upper(package.maintainer),
            //    package.status,
            //));
            packages_count += 1;

            pb.set_message(format!(
                "Found {} new packages mutations...",
                packages_count
            ));
        }

        pb.finish_with_message(format!(
            "Done fetching packages from Hedera blockchain ! ({} packages mutations found)",
            packages_count
        ));
    }

    /**
     * Install package using package_name argument
     */
    pub async fn run(
        &self,
        config_manager: &ConfigManager,
        blockchains_service: &Arc<BlockchainsService>,
        package_managers_service: &PackageManagersService,
    ) {
        debug!("Subcommand install is being run...");

        // First update available packages list

        self.update(blockchains_service).await;

        // Ask which matching package to install

        let package_name = self.package_name.clone().unwrap();
        let package_version = self.package_version.clone().unwrap();

        let matching_packages = blockchains_service
            .find_package(&package_name, &package_version)
            .await;

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("BPM found these matches :")
            .default(0)
            .items(&matching_packages[..])
            .interact()
            .unwrap();

        let selected_package = matching_packages.get(selection).unwrap();
        let package_manager = package_managers_service
            .get_selected_package_manager()
            .await;

        // Check package status

        if selected_package.status < PackageStatus::Outdated {
            error!(
                "This package cannot be installed given its state : {}",
                selected_package.status
            );
            return;
        }

        let full_package_name = format!("{}:{}", selected_package.name, selected_package.version);

        match package_manager
            .install_from_url(&selected_package.archive_url)
            .await
        {
            Ok(_) => {
                info!(
                    "Package {} has been {} !",
                    full_package_name.blue(),
                    "installed".green()
                );
            }
            Err(_) => {
                error!(
                    "Package {} could not be installed",
                    full_package_name.blue()
                )
            }
        }

        debug!("Subcommand install successfully ran !");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
