use crate::{blockchains::blockchain::BlockchainClient, packages::package::Package};

use super::{
    package_document::PackageDocument, package_integrity_document::PackageIntegrityDocument,
    package_integrity_document_builder::PackageIntegrityDocumentBuilder,
};

pub struct PackageDocumentBuilder {
    pub name: Option<String>,
    pub version: Option<String>,
    pub status: Option<i32>,
    pub maintainer: Option<String>,
    pub archive_url: Option<String>,
    pub integrity: Option<PackageIntegrityDocument>,
    pub sig: Option<Vec<u8>>,
    pub blockchain_label: Option<String>,
}

impl PackageDocumentBuilder {
    /**
     * Build from package
     */
    pub fn from_package(package: &Package, blockchain_client: &Box<dyn BlockchainClient>) -> Self {
        let maintainer = hex::encode(package.maintainer.to_bytes());

        let status: u8 = package.status.clone() as u8;

        let integrity =
            PackageIntegrityDocumentBuilder::from_package_integrity(&package.integrity).build();
        let instance = Self {
            name: Some(package.name.clone()),

            version: Some(package.version.clone()),

            status: Some(i32::from(status)),

            maintainer: Some(maintainer),

            archive_url: Some(package.archive_url.to_string()),

            integrity: Some(integrity),

            sig: Some(package.sig.unwrap().to_vec()),

            blockchain_label: Some(blockchain_client.get_label()),
        };

        instance
    }

    /**
     * Reset builder
     */
    pub fn reset(&mut self) -> &mut Self {
        self.name = None;
        self.version = None;
        self.status = None;
        self.maintainer = None;
        self.archive_url = None;
        self.integrity = None;
        self.sig = None;
        self.blockchain_label = None;

        self
    }

    /**
     * Build package document
     */
    pub fn build(&mut self) -> PackageDocument {
        let encoded_sig = hex::encode(&self.sig.clone().expect("Package sig must be set"));

        let doc = PackageDocument {
            name: self.name.clone().expect("Package name must be set"),
            version: self.version.clone().expect("Package version must be set"),
            status: self.status.clone().expect("Package status must be set"),
            maintainer: self
                .maintainer
                .clone()
                .expect("Package maintainer must be set"),
            archive_url: self
                .archive_url
                .clone()
                .expect("Package archive url must be set"),
            integrity: self
                .integrity
                .clone()
                .expect("Package integrity must be set"),
            sig: encoded_sig,
            blockchain_label: self
                .blockchain_label
                .clone()
                .expect("Blockchain label must be set"),
        };

        self.reset();

        doc
    }
}

impl Default for PackageDocumentBuilder {
    fn default() -> Self {
        let instance = Self {
            name: None,
            version: None,
            status: None,
            maintainer: None,
            archive_url: None,
            integrity: None,
            sig: None,
            blockchain_label: None,
        };

        instance
    }
}
