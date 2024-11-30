use std::sync::Arc;

use ed25519_dalek::VerifyingKey;
use log::debug;

use crate::{
    blockchains::blockchain::BlockchainClient,
    db::{
        documents::package_document_builder::PackageDocumentBuilder, traits::repository::Repository,
    },
    packages::{package::Package, package_builder::PackageBuilder},
};

use super::db::packages_repository::PackagesRepository;

/**
 * Packages service
 */
pub struct PackagesService {
    packages_repository: Arc<PackagesRepository>,
}

impl PackagesService {
    /**
     * Add new package to DB
     */
    pub async fn add(&self, package: &Package, blockchain_client: &Box<dyn BlockchainClient>) {
        debug!("Adding new package...");

        let mut builder = PackageDocumentBuilder::from_package(&package, &blockchain_client);

        let package_doc = builder.build();

        self.packages_repository.create(&package_doc).await;

        debug!("Done adding new package !");
    }

    /**
     * Check if package exists
     */
    pub async fn exists(
        &self,
        package: &Package,
        blockchain_client: &Box<dyn BlockchainClient>,
    ) -> bool {
        let doc = PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        let key = self.packages_repository.get_composite_key(&doc);

        let package_exists = self.packages_repository.exists_by_key(&key).await;

        package_exists
    }

    /**
     * Get all packages
     */
    pub async fn get_all(&self) -> Vec<Package> {
        debug!("Getting all packages...");

        let packages: Vec<Package> = self
            .packages_repository
            .read_all()
            .await
            .iter()
            .map(|doc| {
                let package = PackageBuilder::from_document(&doc).build();

                package
            })
            .collect();

        debug!("Done getting all packages !");

        packages
    }

    /**
     * Get by release name
     */
    pub async fn get_by_release(
        &self,
        package_name: &String,
        package_version: &String,
        blockchain_client: &Box<dyn BlockchainClient>,
    ) -> Vec<Package> {
        let packages = self
            .packages_repository
            .read_by_release(
                &package_name,
                &package_version,
                &blockchain_client.get_label(),
            )
            .await
            .iter()
            .map(|doc| {
                let package = PackageBuilder::from_document(&doc).build();

                package
            })
            .collect();

        packages
    }

    /**
     * Get packages by maintainer
     */
    pub async fn get_by_maintainer(
        &self,
        maintainer: &VerifyingKey,
        blockchain_client: &Box<dyn BlockchainClient>,
    ) -> Vec<Package> {
        let encoded_maintainer = hex::encode(maintainer.to_bytes());
        let packages = self
            .packages_repository
            .read_by_maintainer(&encoded_maintainer, &blockchain_client.get_label())
            .await
            .iter()
            .map(|doc| {
                let package = PackageBuilder::from_document(&doc).build();

                package
            })
            .collect();

        packages
    }

    /**
     * Update package
     */
    pub async fn update_package(
        &self,
        package: &Package,
        blockchain_client: &Box<dyn BlockchainClient>,
    ) {
        debug!("Updating package {} from packages service...", package.name);

        let package_doc =
            PackageDocumentBuilder::from_package(&package, &blockchain_client).build();

        let package_doc_key = self.packages_repository.get_composite_key(&package_doc);

        self.packages_repository
            .update(&package_doc_key, &package_doc)
            .await;

        debug!(
            "Done updating package {} from packages service !",
            package.name
        );
    }
}

impl From<&Arc<PackagesRepository>> for PackagesService {
    fn from(value: &Arc<PackagesRepository>) -> Self {
        Self {
            packages_repository: Arc::clone(value),
        }
    }
}

#[cfg(test)]

mod tests {
    use std::sync::Arc;

    use ed25519::signature::rand_core::OsRng;
    use ed25519_dalek::SigningKey;

    use crate::{
        blockchains::blockchain::{BlockchainClient, MockBlockchainClient},
        packages::{package_builder::PackageBuilder, utils::signatures::sign_package},
        services::{db::packages_repository::PackagesRepository, packages::PackagesService},
        test_utils::{
            db::tests::create_test_db,
            package::tests::{create_package_with_sig, create_package_without_sig},
        },
    };

    /**
     * It should add new package
     */
    #[tokio::test]
    async fn test_should_add_package() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);
        let expected_package = create_package_with_sig()?;

        packages_service
            .add(&expected_package, &blockchain_client)
            .await;

        let db_packages = packages_service
            .get_by_release(
                &expected_package.name,
                &expected_package.version,
                &blockchain_client,
            )
            .await;
        assert_eq!(expected_package, db_packages[0]);

        Ok(())
    }

    /**
     * It should get all packages
     */
    #[tokio::test]
    async fn test_should_get_all_packages() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);
        let package_one = create_package_with_sig()?;

        packages_service.add(&package_one, &blockchain_client).await;

        let package_two = create_package_with_sig()?;

        packages_service.add(&package_two, &blockchain_client).await;

        let db_packages = packages_service.get_all().await;

        let expected_packages_count = 2;
        assert_eq!(db_packages.len(), expected_packages_count);

        Ok(())
    }

    /**
     * It should get by maintainer
     */
    #[tokio::test]
    async fn test_should_get_package_by_maintainer() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);

        let mut csprng = OsRng;
        let mut key = SigningKey::generate(&mut csprng);

        let package = create_package_without_sig(&key.verifying_key())?;

        let sig = sign_package(&package, &mut key);

        let signed_package = PackageBuilder::from_package(&package)
            .set_signature(&sig)
            .build();

        packages_service
            .add(&signed_package, &blockchain_client)
            .await;

        let db_packages = packages_service
            .get_by_maintainer(&signed_package.maintainer, &blockchain_client)
            .await;

        let expected_packages_count = 1;

        assert_eq!(db_packages.len(), expected_packages_count);

        Ok(())
    }
}
