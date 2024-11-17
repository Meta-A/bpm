use core::services::package_managers::PackageManagersService;

use colored::Colorize;

use clap::Parser;
use log::{debug, error, info};

/** Remove package using its name */
#[derive(Debug, Parser)]
pub struct RemoveCommand {
    #[clap(required = true)]
    pub package_name: Option<String>,
}

impl RemoveCommand {
    /**
     * Remove package using package_name argument
     */
    pub async fn run(&self, package_managers_service: &PackageManagersService) {
        debug!("Subcommand remove is being run...");

        let package_name = self.package_name.as_ref().unwrap();

        let package_manager = package_managers_service
            .get_selected_package_manager()
            .await;

        // TODO : when fetching by installed implemented use this instead of raw package_name
        match package_manager.remove(package_name).await {
            Ok(_) => {
                info!(
                    "Package {} has been {} !",
                    package_name.blue(),
                    "removed".red()
                );
            }
            Err(_) => {
                error!("Package {} could not be removed", package_name.blue())
            }
        }

        debug!("Subcommand remove successfully ran !");
    }
}
