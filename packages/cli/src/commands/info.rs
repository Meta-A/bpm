use clap::Parser;
use core::{
    blockchains::{
        hedera::blockchain_client::HederaBlockchainClient,
        traits::blockchain_reader::BlockchainReader,
    },
    package_managers::{
        pacman::pacman_package_manager::PacmanPackageManager,
        traits::package_manager::PackageManager,
    },
    packages::package::Package,
};
use log::{debug, info};
/** Display information about given package */
#[derive(Debug, Parser)]
pub struct InfoCommand {
    #[clap(required = true)]
    pub package_name: Option<String>,

    #[clap(required = false)]
    pub version: Option<String>,
}

/**
 * Handles package information request from CLI
 */
impl InfoCommand {
    /**
     * Gather package information using package_name
     */
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Subcommand info is being run...");

        let client = HederaBlockchainClient::new("4991716".to_string())?;

        //client.submit_package().await.unwrap();
        client.fetch_packages().await.unwrap();

        //let manager = PacmanPackageManager {};
        //manager
        //    .fetch_package_content(&Package {
        //        name: "zsh".to_string(),
        //        version: "1.0.0".to_string(),
        //    })
        //    .await?;
        //
        debug!("Subcommand info successfully ran !");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
