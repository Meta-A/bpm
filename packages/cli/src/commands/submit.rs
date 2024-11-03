use clap::Parser;
use colored::*;
use core::{
    config::manager::ConfigManager,
    packages::{
        package::DEFAULT_PACKAGE_STATUS,
        package_builder::PackageBuilder,
        utils::{integrity::compute_package_file_hash, signatures::sign_package},
    },
    services::blockchains::BlockchainsService,
};
use dialoguer::{theme::ColorfulTheme, Confirm};
use log::{debug, info};
use std::{path::PathBuf, sync::Arc};

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
    pub async fn run(
        &self,
        config_manager: &ConfigManager,
        blockchains_service: &Arc<BlockchainsService>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Subcommand submit is being run...");

        let package_name = self.package_name.as_ref().unwrap();
        let package_version = self.package_version.as_ref().unwrap();
        //let sources_directory = self.package_sources_directory.as_ref().unwrap();
        let package_archive_directory =
            PathBuf::from(self.package_archive_directory.as_ref().unwrap());

        // Get maintainer signing key

        let verifying_key = config_manager.get_verifying_key()?;

        // Compute hashes

        let (package_archive_hash, integrity_algorithm) =
            compute_package_file_hash(&package_archive_directory).await?;

        //let package_source_code_hash =
        //    compute_package_file_hash(&package_archive_directory).await?;
        let mut builder = PackageBuilder::new();

        // Build base package
        let package = builder
            .set_name(package_name.to_string())
            .set_version(package_version.to_string())
            .set_status(DEFAULT_PACKAGE_STATUS)
            .set_maintainer(verifying_key)
            .set_integrity(integrity_algorithm, &package_archive_hash)
            .build();

        // Sign package

        let mut signing_key = config_manager.get_signing_key()?;

        let package_sig = sign_package(&package, &mut signing_key);

        let signed_package = PackageBuilder::from_package(&package)
            .set_signature(package_sig)
            .build();

        verifying_key.verify_strict(package.compute_data_integrity().as_slice(), &package_sig)?;

        info!(
            "{}",
            "Following information will be published to the blockchain :"
                .yellow()
                .bold()
        );

        info!("{}", &signed_package);

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to continue?")
            .interact()
            .unwrap()
        {
            info!("Submitting package to blockchain...");
            blockchains_service.submit_package(&signed_package).await;
        } else {
            println!("nevermind then :(");
        }

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
