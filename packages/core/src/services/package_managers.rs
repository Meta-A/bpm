use std::sync::Arc;

use log::debug;

use crate::{
    package_managers::traits::package_manager::PackageManager, types::asynchronous::AsyncMutex,
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
        debug!("Getting selected package manager...");

        let package_managers = self.available_package_managers.lock().await;

        let selected_id = self
            .selected_package_manager
            .lock()
            .await
            .expect("Package manager selected id must be set in order to get package manager");

        let package_manager = package_managers
            .get(selected_id)
            .expect("Could not find package manager");

        debug!("Done getting selected package manager !");

        Arc::clone(package_manager)
    }
}

#[cfg(test)]
mod tests {
    use crate::package_managers::traits::package_manager::{MockPackageManager, PackageManager};

    use super::*;

    #[tokio::test]
    async fn test_should_get_package_manager() {
        let mut package_manager_mock = MockPackageManager::default();

        package_manager_mock
            .expect_get_name()
            .returning(|| String::from("MockPackageManager"));

        let package_manager: Arc<Box<dyn PackageManager>> =
            Arc::new(Box::new(package_manager_mock));

        let expected_package_manager_name = package_manager.get_name();

        let available_package_managers = vec![package_manager];
        let package_managers_service = PackageManagersService::new(&available_package_managers);

        let current_package_manager = package_managers_service
            .get_selected_package_manager()
            .await;

        assert_eq!(
            current_package_manager.get_name(),
            expected_package_manager_name
        );
    }
}
