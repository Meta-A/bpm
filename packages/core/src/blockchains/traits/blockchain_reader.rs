use crate::packages::package::Package;

/**
 * Blockchain Reader allows to read from blockchain
 */
#[trait_variant::make(BlockchainReader: Send)]
pub trait LocalBlockchainReader {
    async fn fetch_packages(&self) -> Result<Vec<Package>, Box<dyn std::error::Error>>;
}
