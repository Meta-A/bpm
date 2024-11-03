use async_trait::async_trait;
use log::debug;
use polodb_core::{
    bson::{doc, Bson},
    Collection, CollectionT,
};
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

        let db_response_result = collection.find_one(doc! {
            "label": key
        });

        let db_response = match db_response_result {
            Ok(res) => res,
            Err(_) => None,
        };

        debug!("Done searching blockchain in repo using key !");

        db_response
    }

    async fn create(&self, document: &BlockchainDocument) {
        debug!("Adding new blockchain to repo...");
        let blockchains_collection = self.db_client.get_blockchains_collection().await;

        blockchains_collection.insert_one(document).unwrap();

        debug!("Done adding new blockchain to repo !");
    }

    async fn update(&self, document: &BlockchainDocument) {
        debug!("Updating blockchain in repo...");

        let blockchains_collection = self.db_client.get_blockchains_collection().await;

        blockchains_collection
            .update_one(
                doc! {
                    "label": &document.label
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
