use std::process::Command;

use log::debug;

use crate::package_managers::{
    pacman::pacman_package_manager::PacmanPackageManager, traits::package_manager::PackageManager,
};

/**
 * Probe available package managers
 */
pub fn probe_package_managers() -> Vec<Box<dyn PackageManager>> {
    debug!("Probing installed package managers...");

    let supported_package_managers = vec!["pacman"];

    let mut available_package_managers = Vec::new();

    for package_manager_cmd in supported_package_managers {
        // Check if package manager installed
        let command_spawn = Command::new(package_manager_cmd).spawn();
        match command_spawn {
            Ok(_) => (),
            Err(_) => {
                debug!(
                    "Package manager {package_manager_cmd} was not found on system, skipping..."
                );
            }
        }

        // If so, build struct then cast to PackageManager trait
        let package_manager: Box<dyn PackageManager> = match package_manager_cmd {
            "pacman" => Box::new(PacmanPackageManager {}),
            _ => panic!("Package manager exists, but does not match any known struct"),
        };

        available_package_managers.push(package_manager);
    }

    debug!(
        "Done probing installed package managers ! ( found {} package managers )",
        available_package_managers.len()
    );

    available_package_managers
}
