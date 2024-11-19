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

        let key = self.packages_repository.get_composed_key(
            &doc.blockchain_label,
            &doc.name,
            &doc.version,
            &doc.maintainer,
        );

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

        self.packages_repository.update(&package_doc).await;

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
