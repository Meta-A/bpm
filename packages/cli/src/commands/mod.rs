mod install;
mod mutate;
mod remove;
mod submit;

use clap::Parser;
use bpm_core::{
    config::manager::ConfigManager,
    services::{blockchains::BlockchainsService, packages::PackagesService},
};
use mutate::MutateCommand;
use remove::RemoveCommand;

use bpm_core::services::package_managers::PackageManagersService;
use dialoguer::{theme::ColorfulTheme, Select};
use install::InstallCommand;
use std::sync::Arc;
use submit::SubmitCommand;

#[derive(Debug, Parser)]
enum BbpmCLIOptions {
    #[clap(name = "install")]
    Install(InstallCommand),

    #[clap(name = "remove")]
    Remove(RemoveCommand),

    #[clap(name = "mutate")]
    Mutate(MutateCommand),

    #[clap(name = "submit")]
    Submit(SubmitCommand),
}

impl BbpmCLIOptions {
    /**
     * Prompt which blockchain to use
     */
    async fn blockchain_prompt(
        &self,
        config_manager: &mut ConfigManager,
        blockchains_service: &Arc<BlockchainsService>,
    ) {
        // TODO: save selection
        let clients = blockchains_service.get_clients();
        let selections = clients.lock().await;
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which blockchain would you like to use ?")
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        blockchains_service.set_client(selection).await;
    }

    /**
     * Code ran when CLI bootstraped
     */
    pub async fn run(
        &self,
        config_manager: &mut ConfigManager,
        blockchains_service: &Arc<BlockchainsService>,
        packages_service: &Arc<PackagesService>,
        package_managers_service: &Arc<PackageManagersService>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.blockchain_prompt(config_manager, &blockchains_service)
            .await;
        match self {
            Self::Install(install) => {
                install
                    .run(
                        &config_manager,
                        &blockchains_service,
                        package_managers_service,
                    )
                    .await
            }
            Self::Remove(remove) => {
                remove.run(package_managers_service).await;
            }
            Self::Mutate(mutate) => {
                mutate
                    .run(&config_manager, &blockchains_service, &packages_service)
                    .await;
            }
            Self::Submit(submit) => submit.run(&config_manager, blockchains_service).await?,
        }

        Ok(())
    }
}

/**
 * Parse CLI args then run chain of commands
 */
#[cfg(not(tarpaulin_include))]
pub async fn bootstrap(
    config_manager: &mut ConfigManager,
    blockchains_service: &Arc<BlockchainsService>,
    packages_service: &Arc<PackagesService>,
    package_managers_service: &Arc<PackageManagersService>,
) -> Result<(), Box<dyn std::error::Error>> {
    use core::services::packages::PackagesService;

    let args = BbpmCLIOptions::parse();

    args.run(
        config_manager,
        blockchains_service,
        packages_service,
        package_managers_service,
    )
    .await?;

    Ok(())
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//}
