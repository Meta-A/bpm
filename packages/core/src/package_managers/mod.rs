use std::{
    process::{Command, Stdio},
    sync::Arc,
};

use log::{debug, error};
use pacman::pacman_package_manager::PacmanPackageManager;
use traits::package_manager::PackageManager;

pub mod errors;
pub mod pacman;
pub mod traits;

/**
 * Probe and init package managers
 */
pub async fn init_package_managers() -> Vec<Arc<Box<dyn PackageManager>>> {
    debug!("Probing installed package managers...");

    let supported_package_managers = vec!["pacman"];

    let mut package_managers: Vec<Arc<Box<dyn PackageManager>>> = vec![];

    for package_manager_cmd in supported_package_managers {
        // Check if package manager installed
        let command_spawn = Command::new(package_manager_cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match command_spawn {
            Ok(_) => (),
            Err(_) => {
                debug!(
                    "Package manager {package_manager_cmd} was not found on system, skipping..."
                );

                continue;
            }
        }

        // If so, build struct then cast to PackageManager trait
        let package_manager: Arc<Box<dyn PackageManager>> = match package_manager_cmd {
            "pacman" => Arc::new(Box::new(PacmanPackageManager::default())),
            _ => {
                error!(
                    "Package manager {} exists, but does not match any known struct",
                    package_manager_cmd
                );

                continue;
            }
        };

        package_managers.push(package_manager);
    }

    debug!(
        "Done probing installed package managers ! ( found {} package managers )",
        package_managers.len()
    );

    package_managers
}
