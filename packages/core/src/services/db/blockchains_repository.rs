use log::debug;
use polodb_core::{bson::doc, CollectionT};
use std::sync::Arc;

use crate::db::{
    client::DbClient, documents::blockchain_document::BlockchainDocument,
    traits::repository::Repository,
};

pub struct BlockchainsRepository {
    db_client: Arc<DbClient>,
}

#[async_trait::async_trait]
impl Repository<BlockchainDocument, String> for BlockchainsRepository {
    async fn read_all(&self) -> Vec<BlockchainDocument> {
        debug!("Reading all blockchains from repo...");
        let collection = self.db_client.get_blockchains_collection().await;

        let cursor = collection.find(doc! {}).run().unwrap();

        let docs = cursor.map(|doc| doc.unwrap()).collect();

        debug!("Done reading all blockchains from repo !");

        docs
    }

    async fn read_by_key(&self, key: &String) -> Option<BlockchainDocument> {
        debug!("Searching blockchain in repo using key...");
        let collection = self.db_client.get_blockchains_collection().await;

        let db_response = collection
            .find_one(doc! {
                "label": key
            })
            .unwrap();

        debug!("Done searching blockchain in repo using key !");

        db_response
    }

    async fn create(&self, document: &BlockchainDocument) {
        debug!("Adding new blockchain to repo...");
        let blockchains_collection = self.db_client.get_blockchains_collection().await;

        blockchains_collection.insert_one(document).unwrap();

        debug!("Done adding new blockchain to repo !");
    }

    async fn update(&self, doc_key: &String, document: &BlockchainDocument) {
        debug!("Updating blockchain in repo...");

        let blockchains_collection = self.db_client.get_blockchains_collection().await;

        blockchains_collection
            .update_one(
                doc! {
                    "label": &doc_key
                },
                doc! {
                "$set": document
                },
            )
            .unwrap();

        debug!("Done updating blockchain in repo !");
    }

    //async fn delete(&self, key: String) -> BlockchainDocument;

    async fn exists_by_key(&self, key: &String) -> bool {
        debug!("Checking if blockchain already exists...");
        let blockchain_result = self.read_by_key(key).await;

        let exists = match blockchain_result {
            Some(_) => true,
            None => false,
        };

        debug!("Done checking if blockchain already exists ! ({})", exists);

        exists
    }
}

impl From<&Arc<DbClient>> for BlockchainsRepository {
    fn from(value: &Arc<DbClient>) -> Self {
        Self {
            db_client: Arc::clone(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::db::{
        client::DbClient, documents::blockchain_document_builder::BlockchainDocumentBuilder,
        traits::repository::Repository,
    };
    use mockall::{mock, predicate::*};
    use tempfile::TempDir;

    use super::*;

    //mock! {
    //    pub BlockchainsRepository {}
    //    #[async_trait::async_trait]
    //    impl Repository<BlockchainDocument, String> for BlockchainsRepository {
    //        async fn read_all(&self) -> Vec<BlockchainDocument>;
    //
    //        async fn read_by_key(&self, key: &String) -> Option<BlockchainDocument>;
    //
    //        async fn create(&self, document: &BlockchainDocument);
    //
    //        async fn update(&self, document: &BlockchainDocument);
    //
    //        async fn exists_by_key(&self, key: &String) -> bool;
    //    }
    //}

    /**
     * It should create blockchain entry
     */
    #[tokio::test]
    async fn test_create_blockchain_entry() {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let expected_blockchain_label = "hedera".to_string();
        let expected_sync_time = "0".to_string();

        let expected_blockchain_doc = BlockchainDocumentBuilder::default()
            .set_label(&expected_blockchain_label)
            .set_last_synchronization(&expected_sync_time)
            .build();

        let blockchain_repo = BlockchainsRepository::from(&db_client);

        blockchain_repo.create(&expected_blockchain_doc).await;

        let actual_blockchain_doc = blockchain_repo
            .read_by_key(&expected_blockchain_doc.label)
            .await
            .unwrap();

        assert_eq!(actual_blockchain_doc, expected_blockchain_doc);
    }

    /**
     * It should read all
     */
    #[tokio::test]
    async fn test_read_all_entries() {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let blockchain_repo = BlockchainsRepository::from(&db_client);

        // Blockchain one
        let blockchain_label_one_mock = "hedera".to_string();
        let sync_time_one_mock = "0".to_string();

        let expected_blockchain_doc_one = BlockchainDocumentBuilder::default()
            .set_label(&blockchain_label_one_mock)
            .set_last_synchronization(&sync_time_one_mock)
            .build();

        blockchain_repo.create(&expected_blockchain_doc_one).await;

        // Blockchain two
        let blockchain_label_two_mock = "iota".to_string();
        let expected_blockchain_doc_two =
            BlockchainDocumentBuilder::from_document(&expected_blockchain_doc_one)
                .set_label(&blockchain_label_two_mock)
                .build();

        blockchain_repo.create(&expected_blockchain_doc_two).await;

        let expected_blockchains = vec![expected_blockchain_doc_one, expected_blockchain_doc_two];

        let blockchains_docs = blockchain_repo.read_all().await;

        assert_eq!(blockchains_docs, expected_blockchains);
    }

    /**
     * It should not find while reading by key
     */
    #[tokio::test]
    async fn test_read_by_key_wrong_key() {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let blockchain_label_mock = "foo".to_string();

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let blockchain_repo = BlockchainsRepository::from(&db_client);

        let blockchain_doc_option = blockchain_repo.read_by_key(&blockchain_label_mock).await;

        assert_eq!(blockchain_doc_option.is_none(), true);
    }

    /**
     * It should update blockchain entry
     */
    #[tokio::test]
    async fn test_update_blockchain_entry() {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let expected_sync_time = "123".to_string();

        let mock_blockchain_label = "foo".to_string();

        let mock_sync_time = "0".to_string();

        let mock_blockchain_doc = BlockchainDocumentBuilder::default()
            .set_label(&mock_blockchain_label)
            .set_last_synchronization(&mock_sync_time)
            .build();

        // Create blockchain doc
        let blockchain_repo = BlockchainsRepository::from(&db_client);

        blockchain_repo.create(&mock_blockchain_doc).await;

        // Update blockchain doc
        let updated_blockchain_doc = BlockchainDocumentBuilder::from_document(&mock_blockchain_doc)
            .set_last_synchronization(&expected_sync_time)
            .build();

        blockchain_repo
            .update(&mock_blockchain_label, &updated_blockchain_doc)
            .await;

        let actual_blockchain_doc = blockchain_repo
            .read_by_key(&mock_blockchain_doc.label)
            .await
            .unwrap();

        assert_eq!(actual_blockchain_doc, updated_blockchain_doc);
    }

    /**
     * It should exist using key
     */
    #[tokio::test]
    async fn test_exists_by_key_entry() {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let expected_exists = true;
        let blockchain_label_mock = "hedera".to_string();
        let sync_time_mock = "0".to_string();

        let expected_blockchain_doc = BlockchainDocumentBuilder::default()
            .set_label(&blockchain_label_mock)
            .set_last_synchronization(&sync_time_mock)
            .build();

        let blockchain_repo = BlockchainsRepository::from(&db_client);

        blockchain_repo.create(&expected_blockchain_doc).await;

        let blockchain_doc_exists = blockchain_repo
            .exists_by_key(&expected_blockchain_doc.label)
            .await;

        assert_eq!(blockchain_doc_exists, expected_exists);
    }

    /**
     * It should not exist using key
     */
    #[tokio::test]
    async fn test_does_not_exist_by_key_entry() {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let expected_exists = false;

        let blockchain_label_mock = "foobar".to_string();

        let blockchain_repo = BlockchainsRepository::from(&db_client);

        let blockchain_doc_exists = blockchain_repo.exists_by_key(&blockchain_label_mock).await;

        assert_eq!(blockchain_doc_exists, expected_exists);
    }
}
