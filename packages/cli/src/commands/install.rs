use clap::Parser;
use log::{debug, info};

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
     * Installs package using package_name argument
     */
    pub async fn run(&self) {
        debug!("Subcommand install is being run...");

        let package_name = self.package_name.as_ref().unwrap();

        info!("Installing package {:?}", package_name);

        info!("Done installing package {:?}", package_name);

        debug!("Subcommand install successfully ran !");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
     * It should install package
     */
    #[test]
    fn test_package_installation() {
        let package_name_mock = String::from("foo");
        let command = InstallCommand {
            package_name: Some(package_name_mock),
            version: None,
        };

        command.run();
    }
}
