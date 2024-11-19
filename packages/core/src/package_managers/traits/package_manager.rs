use std::path::PathBuf;

use url::Url;

use crate::package_managers::errors::package_manager_error::PackageManagerError;

#[async_trait::async_trait]
pub trait PackageManager {
    async fn install_from_url(&self, package_url: &Url) -> Result<PathBuf, PackageManagerError>;

    // TODO : When feature to fetch installed packages implement use Package object instead
    async fn remove(&self, package_name: &String) -> Result<(), PackageManagerError>;
}
