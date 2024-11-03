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

                continue;
            }
        }

        // If so, build struct then cast to PackageManager trait
        let package_manager: Box<dyn PackageManager> = match package_manager_cmd {
            "pacman" => Box::new(PacmanPackageManager {}),
            _ => panic!(
                "Package manager {} exists, but does not match any known struct",
                package_manager_cmd
            ),
        };

        available_package_managers.push(package_manager);
    }

    debug!(
        "Done probing installed package managers ! ( found {} package managers )",
        available_package_managers.len()
    );

    available_package_managers
}

mod tests {
    use std::{
        env,
        fs::{self, File},
        io::Write,
        os::unix::fs::PermissionsExt,
    };

    use tempfile::TempDir;

    use super::probe_package_managers;

    #[test]
    fn test_should_probe_package_managers() {
        let test_dir = TempDir::new().unwrap();

        let expected_package_manager = "apt";
        let package_manager_dir = test_dir.path().join(expected_package_manager);

        // Create the file
        let mut file = File::create(&package_manager_dir).unwrap();

        // Set permissions to make the file executable
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&package_manager_dir, permissions).unwrap();

        let content = "echo 'APT package manager';";

        // Write the string to the file
        file.write_all(content.as_bytes()).unwrap();

        let original_path = env::var("PATH").unwrap();

        let new_path_var = format!("{}:{}", test_dir.path().to_str().unwrap(), original_path);

        env::set_var("PATH", new_path_var);

        let package_managers = probe_package_managers();
    }
}
