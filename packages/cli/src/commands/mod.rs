mod install;
mod submit;

use clap::Parser;
use core::{
    blockchains::{blockchain::BlockchainClient, hedera::blockchain_client::HederaBlockchain},
    config::manager::ConfigManager,
    db::client::DbClient,
    services::blockchains::BlockchainsService,
};
use dialoguer::{theme::ColorfulTheme, Select};
use install::InstallCommand;
use std::{rc::Rc, sync::Arc};
use submit::SubmitCommand;
use tokio::sync::Mutex;

#[derive(Debug, Parser)]
enum BbpmCLIOptions {
    #[clap(name = "install")]
    Install(InstallCommand),

    #[clap(name = "submit")]
    Submit(SubmitCommand),
}

impl BbpmCLIOptions {
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

    pub async fn run(
        &self,
        config_manager: &mut ConfigManager,
        blockchains_service: &Arc<BlockchainsService>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.blockchain_prompt(config_manager, &blockchains_service)
            .await;
        match self {
            Self::Install(install) => install.run(&config_manager, &blockchains_service).await,
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
) -> Result<(), Box<dyn std::error::Error>> {
    let args = BbpmCLIOptions::parse();

    args.run(config_manager, blockchains_service).await?;

    Ok(())
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    /**
//     * It should run package installation command
//     */
//    #[test]
//    fn test_run_package_installation() {
//        let bppm_cli = BbpmCLIOptions::parse_from(vec!["bbpm", "install", "foobar"]);
//
//        let command = bppm_cli.run();
//
//        assert!(matches!(command, BbpmCLIOptions::Install { .. }));
//    }
//}
