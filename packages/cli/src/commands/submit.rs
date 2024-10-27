use clap::Parser;
use core::{
    blockchains::{
        hedera::blockchain_client::HederaBlockchainClient,
        traits::blockchain_writer::BlockchainWriter,
    },
    packages::{
        package_builder::PackageBuilder,
        utils::integrity::{compute_package_archive_hash, compute_package_binaries_hashes},
    },
};
use log::{debug, info};
use std::path::PathBuf;

/** Submit package using sources  */
#[derive(Debug, Parser)]
pub struct SubmitCommand {
    /**
     * Package name ( eg: neofetch )
     */
    #[clap(required = true)]
    pub package_name: Option<String>,

    /**
     * Package version ( eg: 7.1.0-2  )
     */
    #[clap(required = true)]
    pub package_version: Option<String>,

    /**
     * Package sources directory ( eg: git repo... )
     */
    #[clap(required = true)]
    pub package_sources_directory: Option<String>,

    /**
     * Package archive directory ( eg: neofetch-7.1.0-2-any.pkg.tar.zst... )
     */
    #[clap(required = true)]
    pub package_archive_directory: Option<String>,
}

/**
 * Handle package submission request from CLI
 */
impl SubmitCommand {
    /**
     * Submit command
     */
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Subcommand submit is being run...");

        let blockchain: Box<dyn BlockchainWriter> =
            Box::new(HederaBlockchainClient::new("4991716".to_string())?);
        let package_name = self.package_name.as_ref().unwrap();
        let package_version = self.package_version.as_ref().unwrap();
        //let sources_directory = self.package_sources_directory.as_ref().unwrap();
        let package_archive_directory =
            PathBuf::from(self.package_archive_directory.as_ref().unwrap());

        // TODO: Bad find way to dynamically pass hasher
        let integrity_alg = "SHA256".to_string();

        let package_content_hash = compute_package_archive_hash(package_archive_directory).await?;

        let mut builder = PackageBuilder::new();

        let package = builder
            .set_package_name(package_name.to_string())
            .set_package_version(package_version.to_string())
            .set_package_integrity(integrity_alg, package_content_hash)
            .build();

        blockchain.submit_package(&package).await?;

        debug!("Subcommand submit successfully ran !");

        Ok(())
    }
}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    /**
//     * It should install package
//     */
//    #[test]
//    fn test_package_installation() {
//        let package_name_mock = String::from("foo");
//        let command = InstallCommand {
//            package_name: Some(package_name_mock),
//            version: None,
//        };
//
//        command.run();
//    }
//}
