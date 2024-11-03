use core::config::manager::ConfigManager;
use core::services::blockchains::BlockchainsService;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use tokio::sync::mpsc;

/** Install package using its name */
#[derive(Debug, Parser)]
pub struct InstallCommand {
    #[clap(required = true)]
    pub package_name: Option<String>,

    #[clap(required = false)]
    pub version: Option<String>,
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
     * Install package using package_name argument
     */
    pub async fn run(
        &self,
        config_manager: &ConfigManager,
        blockchains_service: &Arc<BlockchainsService>,
    ) {
        debug!("Subcommand install is being run...");

        let (tx_packages_update, mut rx_packages_update) = mpsc::channel(1);

        let task_blockchains_service_ref = Arc::clone(&blockchains_service);
        tokio::spawn(async move {
            task_blockchains_service_ref
                .update(&tx_packages_update)
                .await;
        });

        let mut packages_count: u128 = 0;

        let pb = self.build_progress_bar();

        while let Some(package) = rx_packages_update.recv().await {
            pb.println(format!(
                "New package found => {}:{} ( Maintainer : {}, Status : {}  )",
                package.name,
                package.version,
                hex::encode_upper(package.maintainer),
                package.status,
            ));
            packages_count += 1;

            pb.set_message(format!("Found {} packages...", packages_count));
        }

        pb.finish_with_message(format!(
            "Done fetching packages from Hedera blockchain ! ({} packages found)",
            packages_count
        ));

        debug!("Subcommand install successfully ran !");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //
    // It should install package
    //
    //#[test]
    //fn test_package_installation() {
    //    let package_name_mock = String::from("foo");
    //    let command = InstallCommand {
    //        package_name: Some(package_name_mock),
    //        version: None,
    //    };
    //
    //    command.run();
    //}
}
