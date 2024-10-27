mod info;
mod install;
mod submit;

use clap::Parser;
use core::config::manager::ConfigManager;
use info::InfoCommand;
use install::InstallCommand;
use submit::SubmitCommand;
#[derive(Debug, Parser)]
enum BbpmCLIOptions {
    #[clap(name = "install")]
    Install(InstallCommand),

    #[clap(name = "info")]
    Info(InfoCommand),

    #[clap(name = "submit")]
    Submit(SubmitCommand),
}

impl BbpmCLIOptions {
    pub async fn run(
        &self,
        config_manager: &mut ConfigManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Install(install) => install.run().await,
            Self::Info(info) => info.run().await?,
            Self::Submit(submit) => submit.run().await?,
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
) -> Result<(), Box<dyn std::error::Error>> {
    let args = BbpmCLIOptions::parse();

    args.run(config_manager).await?;

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
