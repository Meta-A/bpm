mod install;

use clap::Parser;
use install::InstallCommand;

#[derive(Debug, Parser)]
enum BbpmCLIOptions {
    #[clap(name = "install")]
    Install(InstallCommand),
}

impl BbpmCLIOptions {
    pub fn run(&self) -> &Self {
        match self {
            Self::Install(install) => install.run(),
        }

        self
    }
}

/**
 * Parse CLI args then run chain of commands
 */
#[cfg(not(tarpaulin_include))]
pub fn bootstrap() {
    let args = BbpmCLIOptions::parse();

    args.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
     * It should run package installation command
     */
    #[test]
    fn test_run_package_installation() {
        let bppm_cli = BbpmCLIOptions::parse_from(vec!["bbpm", "install", "foobar"]);

        let command = bppm_cli.run();

        assert!(matches!(command, BbpmCLIOptions::Install { .. }));
    }
}
