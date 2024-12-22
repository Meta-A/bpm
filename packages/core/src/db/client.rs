use std::{path::PathBuf, sync::Arc};

use polodb_core::{Collection, Database};
use tokio::sync::Mutex;

use super::documents::{
    blockchain_document::BlockchainDocument, package_document::PackageDocument,
};

pub struct DbClient {
    instance: Arc<Mutex<Database>>,
}

impl DbClient {
    /**
     * Get packages collection
     */
    pub async fn get_packages_collection(&self) -> Collection<PackageDocument> {
        let packages_collection_name: &str = "packages";

        let packages_collection = self
            .instance
            .lock()
            .await
            .collection(packages_collection_name);

        packages_collection
    }

    /**
     * Get blockchains collection
     */
    pub async fn get_blockchains_collection(&self) -> Collection<BlockchainDocument> {
        let blockchains_collection_name: &str = "blockchains";

        let blockchains_collection = self
            .instance
            .lock()
            .await
            .collection(blockchains_collection_name);

        blockchains_collection
    }
}

impl From<&PathBuf> for DbClient {
    /**
     * New instance from DB path
     */
    fn from(db_path: &PathBuf) -> Self {
        let db = Arc::new(Mutex::new(Database::open_path(db_path).unwrap()));

        let instance = Self { instance: db };

        instance
    }
}

#[cfg(test)]
mod tests {
    use polodb_core::CollectionT;
    use tempfile::TempDir;

    use super::*;

    /**
     * It should initialize DB
     */
    #[test]
    fn test_db_init() {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let _ = DbClient::from(&test_dir_path);

        assert_eq!(test_dir_path.exists(), true);
    }

    /**
     * It should get packages collection
     */
    #[tokio::test]
    async fn test_get_packages_collection() -> Result<(), Box<dyn std::error::Error>> {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let client = DbClient::from(&test_dir_path);

        let collection = client.get_packages_collection().await;

        let expected_items_count = 0;
        let items_count = collection.count_documents()?;

        assert_eq!(items_count, expected_items_count);

        Ok(())
    }

    /**
     * It should get blockchains collection
     */
    #[tokio::test]
    async fn test_get_blockchains_collection() -> Result<(), Box<dyn std::error::Error>> {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let client = DbClient::from(&test_dir_path);

        let collection = client.get_blockchains_collection().await;

        let expected_items_count = 0;

        let items_count = collection.count_documents()?;

        assert_eq!(items_count, expected_items_count);

        Ok(())
    }
}
