use bpm_core::config::manager::ConfigManager;
use bpm_core::packages::package::PackageStatus;
use bpm_core::packages::package_builder::PackageBuilder;
use bpm_core::packages::utils::signatures::sign_package;
use bpm_core::services::blockchains::BlockchainsService;
use bpm_core::services::packages::PackagesService;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use std::str::FromStr;
use strum::IntoEnumIterator;

/** Mutate package */
#[derive(Debug, Parser)]
pub struct MutateCommand {}

/**
 * Handles package mutation request from CLI
 */
impl MutateCommand {
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
     * Install package using package_name argument
     */
    pub async fn run(
        &self,
        config_manager: &ConfigManager,
        blockchains_service: &Arc<BlockchainsService>,
        packages_service: &PackagesService,
    ) {
        debug!("Subcommand mutate is being run...");

        let maintainer_verifying_key = config_manager
            .get_verifying_key()
            .expect("Could not find maintainer key to mutate package");

        let blockchain_client = blockchains_service.get_selected_client().await;

        let published_packages = packages_service
            .get_by_maintainer(&maintainer_verifying_key, &blockchain_client)
            .await;

        let package_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Published packages")
            .default(0)
            .items(&published_packages[..])
            .interact()
            .unwrap();

        let selected_package = published_packages
            .get(package_selection)
            .expect("Selected package does not exist");

        let package_status_choices: Vec<String> = PackageStatus::iter()
            .map(|status| status.to_string())
            .collect();

        let package_status_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Package status")
            .default(0)
            .items(&package_status_choices[..])
            .interact()
            .unwrap();

        let raw_selected_status = package_status_choices
            .get(package_status_selection)
            .unwrap();
        let selected_status: PackageStatus = PackageStatus::from_str(&raw_selected_status)
            .expect("Could not parse package status from string");

        let updated_package = PackageBuilder::from_package(&selected_package)
            .set_status(&selected_status)
            .build();

        // Sign package

        info!("Signing package mutations...");
        let mut signing_key = config_manager
            .get_signing_key()
            .expect("Could not load your signing key");

        let package_sig = sign_package(&updated_package, &mut signing_key);

        let signed_updated_package = PackageBuilder::from_package(&updated_package)
            .set_signature(&package_sig)
            .build();

        info!("Done signing package mutations !");

        info!("Mutating package remotely...");

        blockchains_service
            .submit_package(&signed_updated_package)
            .await;

        info!("Done mutating package remotely !");

        debug!("Subcommand mutate successfully ran !");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
