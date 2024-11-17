use std::{
    process::{Command, Stdio},
    sync::Arc,
};

use log::debug;
use tokio::sync::Mutex;

use crate::package_managers::{
    pacman::pacman_package_manager::PacmanPackageManager, traits::package_manager::PackageManager,
};

pub struct PackageManagersService {
    available_package_managers: Arc<Mutex<Vec<Arc<Box<dyn PackageManager>>>>>,
    selected_package_manager: Arc<Mutex<Option<usize>>>,
}

impl PackageManagersService {
    /**
     * Create service
     */
    pub fn new() -> Self {
        Self {
            available_package_managers: Arc::new(Mutex::new(Vec::new())),
            selected_package_manager: Arc::new(Mutex::new(Some(0))),
        }
    }
    /**
     * Probe and init package managers
     */
    pub async fn init_package_managers(&self) {
        debug!("Probing installed package managers...");

        let supported_package_managers = vec!["pacman"];

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
                "pacman" => Arc::new(Box::new(PacmanPackageManager {})),
                _ => panic!(
                    "Package manager {} exists, but does not match any known struct",
                    package_manager_cmd
                ),
            };

            self.available_package_managers
                .lock()
                .await
                .push(package_manager);
        }

        debug!(
            "Done probing installed package managers ! ( found {} package managers )",
            self.available_package_managers.lock().await.len()
        );
    }

    /**
     * Return selected package manager
     */
    pub async fn get_selected_package_manager(&self) -> Arc<Box<dyn PackageManager>> {
        let package_managers = self.available_package_managers.lock().await;

        let selected_id = self
            .selected_package_manager
            .lock()
            .await
            .expect("Package manager selected id must be set in order to get package manager");

        let package_manager = package_managers
            .get(selected_id)
            .expect("Could not find package manager");

        Arc::clone(package_manager)
    }
}
