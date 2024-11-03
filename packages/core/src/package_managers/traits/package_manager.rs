use std::path::PathBuf;

use crate::packages::package::Package;
use async_trait::async_trait;

#[async_trait]
pub trait PackageManager {
    async fn fetch_package_archive(
        &self,
        package_name: &String,
    ) -> Result<PathBuf, Box<dyn std::error::Error>>;
}
