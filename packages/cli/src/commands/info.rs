use clap::Parser;
use core::blockchains::{
    abstractions::blockchain_client::BlockchainClient,
    hedera::blockchain_client::HederaBlockchainClient,
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
    pub async fn run(&self) {
        debug!("Subcommand info is being run...");

        let network_address = "https://testnet.mirrornode.hedera.com:443";

        let mut client = HederaBlockchainClient::from(network_address.to_string());

        client
            .with_topic(4991716, 0, 0)
            .get_packages()
            .await
            .unwrap();
        //
        debug!("Subcommand info successfully ran !");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
