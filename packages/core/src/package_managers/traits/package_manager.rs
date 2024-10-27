use crate::packages::package::Package;
use async_trait::async_trait;

#[async_trait]
pub trait PackageManager {
    async fn fetch_package_content(
        &self,
        package: &Package,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
