mod info;
mod install;

use clap::Parser;
use info::InfoCommand;
use install::InstallCommand;

#[derive(Debug, Parser)]
enum BbpmCLIOptions {
    #[clap(name = "install")]
    Install(InstallCommand),

    #[clap(name = "info")]
    Info(InfoCommand),
}

impl BbpmCLIOptions {
    pub async fn run(&self) -> &Self {
        match self {
            Self::Install(install) => install.run().await,
            Self::Info(info) => info.run().await,
        }

        self
    }
}

/**
 * Parse CLI args then run chain of commands
 */
#[cfg(not(tarpaulin_include))]
pub async fn bootstrap() {
    let args = BbpmCLIOptions::parse();

    args.run().await;
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
