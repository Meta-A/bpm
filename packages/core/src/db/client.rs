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
