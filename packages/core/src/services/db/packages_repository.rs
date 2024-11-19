use log::debug;
use polodb_core::{bson::doc, CollectionT};
use std::sync::Arc;

use crate::db::{
    client::DbClient, documents::package_document::PackageDocument, traits::repository::Repository,
};

pub struct PackagesRepository {
    db_client: Arc<DbClient>,
}

const COMPOSED_KEY_SEPARATOR: &str = ":";

impl PackagesRepository {
    /**
     * Get composed key parts
     * Composed key is currently -> blockchain_label:package_name:package_version:maintainer_key
     */
    fn get_composed_key_parts(&self, key: &String) -> (String, String, String, String) {
        let splitted_key: Vec<&str> = key.split(COMPOSED_KEY_SEPARATOR).collect();

        let blockchain_label = String::from(splitted_key[0]);
        let package_name = String::from(splitted_key[1]);
        let package_version = String::from(splitted_key[2]);
        let maintainer_key = String::from(splitted_key[3]);

        (
            blockchain_label,
            package_name,
            package_version,
            maintainer_key,
        )
    }

    /**
     * Return Bson expected types from provided key
     */
    fn get_bson_composed_key_parts(&self, key: &String) -> (String, String, String, String) {
        let (blockchain_label, package_name, package_version, maintainer_key) =
            self.get_composed_key_parts(&key);

        (
            blockchain_label,
            package_name,
            package_version,
            maintainer_key,
        )
    }

    /**
     * Create unique composed key
     */
    pub fn get_composed_key(
        &self,
        blockchain_label: &String,
        package_name: &String,
        package_version: &String,
        maintainer_key: &String,
    ) -> String {
        let key = format!("{blockchain_label}:{package_name}:{package_version}:{maintainer_key}");

        key
    }

    /**
     * Find packages in given blockcahin by release name
     */
    pub async fn read_by_release(
        &self,
        package_name: &String,
        package_version: &String,
        blockchain_label: &String,
    ) -> Vec<PackageDocument> {
        debug!("Searching packages in repo using name {}...", package_name);
        let collection = self.db_client.get_packages_collection().await;

        let cursor = collection
            .find(doc! {
                "name": package_name,
                "version": package_version,
                "blockchain_label": blockchain_label,

            })
            .run()
            .unwrap();

        let docs = cursor.map(|doc| doc.unwrap()).collect();

        debug!("Done searching packages with name {} !", package_name);

        docs
    }

    /**
     * Read by maintainer
     */

    pub async fn read_by_maintainer(
        &self,
        maintainer: &String,
        blockchain_label: &String,
    ) -> Vec<PackageDocument> {
        debug!(
            "Searching packages in repo using maintainer {}...",
            maintainer
        );
        let collection = self.db_client.get_packages_collection().await;

        let cursor = collection
            .find(doc! {
                "maintainer": maintainer,
                "blockchain_label": blockchain_label,

            })
            .run()
            .unwrap();

        let docs = cursor.map(|doc| doc.unwrap()).collect();

        debug!("Done searching packages with maintainer {} !", maintainer);

        docs
    }
}

#[async_trait::async_trait]
impl Repository<PackageDocument, String> for PackagesRepository {
    async fn read_all(&self) -> Vec<PackageDocument> {
        debug!("Reading all packages from repo...");

        let collection = self.db_client.get_packages_collection().await;

        let cursor = collection.find(doc! {}).run().unwrap();

        let docs = cursor.map(|doc| doc.unwrap()).collect();

        debug!("Done reading all packages from repo !");

        docs
    }

    /**
     * Read document by key
     */
    async fn read_by_key(&self, key: &String) -> Option<PackageDocument> {
        debug!("Searching package {} in repo using key...", key);
        let collection = self.db_client.get_packages_collection().await;

        let (blockchain_label, package_name, package_version, maintainer_key) =
            self.get_bson_composed_key_parts(key);

        let db_response_result = collection.find_one(doc! {
            "name": package_name,
            "version": package_version,
            "maintainer": maintainer_key,
            "blockchain_label": blockchain_label,

        });

        let db_response = match db_response_result {
            Ok(res) => res,
            Err(_) => None,
        };

        debug!("Done searching blockchain in repo using key !");

        db_response
    }

    /**
     * Create package document
     */
    async fn create(&self, document: &PackageDocument) {
        debug!("Adding new package to repo...");
        let collection = self.db_client.get_packages_collection().await;

        collection.insert_one(document).unwrap();

        debug!("Done adding new package to repo !");
    }

    /**
     * Update package document
     */
    async fn update(&self, document: &PackageDocument) {
        debug!("Updating package in repo...");

        let collection = self.db_client.get_packages_collection().await;

        let composed_key = self.get_composed_key(
            &document.blockchain_label,
            &document.name,
            &document.version,
            &document.maintainer,
        );

        let (blockchain_label, package_name, package_version, maintainer_key) =
            self.get_bson_composed_key_parts(&composed_key);

        collection
            .update_one(
                doc! {
                "name": package_name,
                "version": package_version,
                "maintainer": maintainer_key,
                "blockchain_label": blockchain_label,

                    },
                doc! {
                "$set": document
                },
            )
            .unwrap();

        debug!("Done updating package in repo !");
    }
    //async fn delete(&self, key: String) -> BlockchainDocument;
    /**
     * Check if exists by key
     */
    async fn exists_by_key(&self, key: &String) -> bool {
        debug!("Checking if package already exists...");
        let blockchain_result = self.read_by_key(key).await;

        let exists = match blockchain_result {
            Some(_) => true,
            None => false,
        };

        debug!("Done checking if package already exists ! ({})", exists);

        exists
    }
}

impl From<&Arc<DbClient>> for PackagesRepository {
    fn from(value: &Arc<DbClient>) -> Self {
        Self {
            db_client: Arc::clone(value),
        }
    }
}
