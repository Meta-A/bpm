use crate::packages::package::Package;

/**
 * Blockchain Reader allows to write to blockchain
 */
#[async_trait::async_trait]
pub trait BlockchainWriter {
    async fn submit_package(&self, package: &Package) -> Result<(), Box<dyn std::error::Error>>;
}
