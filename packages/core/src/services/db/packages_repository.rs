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
    fn get_composite_key_parts(&self, key: &String) -> (String, String, String, String) {
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
     * Create unique composed key
     */
    pub fn get_composite_key(&self, document: &PackageDocument) -> String {
        let key = format!(
            "{}:{}:{}:{}",
            document.blockchain_label, document.name, document.version, document.maintainer
        );

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
            self.get_composite_key_parts(key);

        let db_response = collection
            .find_one(doc! {
                "name": package_name,
                "version": package_version,
                "maintainer": maintainer_key,
                "blockchain_label": blockchain_label,

            })
            .unwrap();

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
    async fn update(&self, doc_composite_key: &String, document: &PackageDocument) {
        debug!("Updating package in repo...");

        let collection = self.db_client.get_packages_collection().await;

        let (blockchain_label, package_name, package_version, maintainer_key) =
            self.get_composite_key_parts(&doc_composite_key);

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        blockchains::{blockchain::BlockchainClient, hedera::blockchain_client::HederaBlockchain},
        db::{
            client::DbClient, documents::package_document_builder::PackageDocumentBuilder,
            traits::repository::Repository,
        },
        packages::package_status::PackageStatus,
        test_utils::package::tests::create_package_with_sig,
    };
    use tempfile::TempDir;

    use super::*;

    /**
     * It should create package entry
     */
    #[tokio::test]
    async fn test_create_package_entry() {
        let package = create_package_with_sig().unwrap();

        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let packages_repo = PackagesRepository::from(&db_client);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        let expected_package_doc =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        packages_repo.create(&expected_package_doc).await;

        let expected_package_doc_key = &packages_repo.get_composite_key(&expected_package_doc);

        let actual_package_doc = packages_repo
            .read_by_key(&expected_package_doc_key)
            .await
            .unwrap();

        assert_eq!(actual_package_doc, expected_package_doc);
    }

    /**
     * It should return None if package not found
     */
    #[tokio::test]
    async fn test_package_not_found_by_key() {
        let package = create_package_with_sig().unwrap();
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let packages_repo = PackagesRepository::from(&db_client);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        // do not insert it in db
        let package_doc =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        let key = packages_repo.get_composite_key(&package_doc);

        let package_doc_opt = packages_repo.read_by_key(&key).await;

        assert_eq!(package_doc_opt.is_none(), true);
    }

    /**
     * It should read by release
     */
    #[tokio::test]
    async fn test_read_by_release_entry() {
        let package = create_package_with_sig().unwrap();

        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let packages_repo = PackagesRepository::from(&db_client);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        let expected_package_doc =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        packages_repo.create(&expected_package_doc).await;

        let packages_docs = packages_repo
            .read_by_release(
                &package.name,
                &package.version,
                &blockchain_client.get_label(),
            )
            .await;

        assert_eq!(packages_docs[0], expected_package_doc);
    }

    /**
     * It should read by maintainer
     */
    #[tokio::test]
    async fn test_read_by_maintainer_entry() {
        let package = create_package_with_sig().unwrap();

        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let packages_repo = PackagesRepository::from(&db_client);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        let expected_package_doc =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        packages_repo.create(&expected_package_doc).await;

        let packages_docs = packages_repo
            .read_by_maintainer(
                &expected_package_doc.maintainer,
                &blockchain_client.get_label(),
            )
            .await;

        assert_eq!(packages_docs[0], expected_package_doc);
    }

    /**
     * It should read all packages entries
     */
    #[tokio::test]
    async fn test_read_all_packages_entries() {
        let package = create_package_with_sig().unwrap();

        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let packages_repo = PackagesRepository::from(&db_client);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        let expected_package_doc_one =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        let expected_package_doc_two_mock = "bar".to_string();
        let expected_package_doc_two =
            PackageDocumentBuilder::from_package(&package, &blockchain_client)
                .set_name(&expected_package_doc_two_mock)
                .build();

        packages_repo.create(&expected_package_doc_one).await;
        packages_repo.create(&expected_package_doc_two).await;

        let expected_packages_docs = vec![expected_package_doc_one, expected_package_doc_two];

        let packages_docs = packages_repo.read_all().await;

        assert_eq!(packages_docs, expected_packages_docs);
    }

    /**
     * It should update package entry
     */
    #[tokio::test]
    async fn test_update_package_entry() {
        let package = create_package_with_sig().unwrap();

        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let packages_repo = PackagesRepository::from(&db_client);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        // Create package entry
        let package_doc_mock =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        packages_repo.create(&package_doc_mock).await;

        let package_doc_mock_key = &packages_repo.get_composite_key(&package_doc_mock);

        // Update package entry

        let expected_status = PackageStatus::Outdated;
        let expected_package_doc = PackageDocumentBuilder::from_document(&package_doc_mock)
            .set_status(&expected_status)
            .build();

        packages_repo
            .update(package_doc_mock_key, &expected_package_doc)
            .await;

        let expected_package_doc_key = &packages_repo.get_composite_key(&expected_package_doc);

        let actual_package_doc = packages_repo
            .read_by_key(&expected_package_doc_key)
            .await
            .unwrap();

        assert_eq!(actual_package_doc.status, i32::from(expected_status as u8));
    }

    /**
     * It should exist by composite key
     */
    #[tokio::test]
    async fn test_should_exist_by_composite_key() {
        let package = create_package_with_sig().unwrap();

        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let packages_repo = PackagesRepository::from(&db_client);

        let expected_exists = true;

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        let package_doc =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        packages_repo.create(&package_doc).await;

        let expected_package_doc_key = packages_repo.get_composite_key(&package_doc);

        let exists = packages_repo.exists_by_key(&expected_package_doc_key).await;

        assert_eq!(exists, expected_exists);
    }

    /**
     * It should not exist by composite key
     */
    #[tokio::test]
    async fn test_should_not_exist_by_composite_key() {
        let package = create_package_with_sig().unwrap();

        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        let packages_repo = PackagesRepository::from(&db_client);

        let expected_exists = false;

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        // do not insert package_doc
        let package_doc =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        let expected_package_doc_key = packages_repo.get_composite_key(&package_doc);

        let exists = packages_repo.exists_by_key(&expected_package_doc_key).await;

        assert_eq!(exists, expected_exists);
    }
}
