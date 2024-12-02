use std::path::PathBuf;

use url::Url;

use crate::package_managers::errors::package_manager_error::PackageManagerError;

#[cfg(test)]
use mockall::automock;

#[async_trait::async_trait]
#[cfg_attr(test, automock)]
pub trait PackageManager {

    fn get_name(&self) -> String;

    async fn install_from_url(&self, package_url: &Url) -> Result<PathBuf, PackageManagerError>;

    // TODO : When feature to fetch installed packages implement use Package object instead
    async fn remove(&self, package_name: &String) -> Result<(), PackageManagerError>;
}
