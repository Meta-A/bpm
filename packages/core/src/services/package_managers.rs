use std::{
    process::{Command, Stdio},
    sync::Arc,
};

use log::debug;

use crate::{
    package_managers::{
        pacman::pacman_package_manager::PacmanPackageManager,
        traits::package_manager::PackageManager,
    },
    types::asynchronous::AsyncMutex,
};

/**
 * Package managers service
 */
pub struct PackageManagersService {
    available_package_managers: Arc<AsyncMutex<Vec<Arc<Box<dyn PackageManager>>>>>,
    selected_package_manager: Arc<AsyncMutex<Option<usize>>>,
}

impl PackageManagersService {
    /**
     * Create service
     */
    pub fn new(available_package_managers: &Vec<Arc<Box<dyn PackageManager>>>) -> Self {
        Self {
            available_package_managers: Arc::new(AsyncMutex::new(
                available_package_managers.clone(),
            )),
            selected_package_manager: Arc::new(AsyncMutex::new(Some(0))),
        }
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
