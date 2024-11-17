use crate::packages::package_integrity::PackageIntegrity;

use super::package_integrity_document::PackageIntegrityDocument;

pub struct PackageIntegrityDocumentBuilder {
    algorithm: Option<String>,
    archive_hash: Option<Vec<u8>>,
}

impl PackageIntegrityDocumentBuilder {
    /**
     * Build from package integrity
     */
    pub fn from_package_integrity(package_integrity: &PackageIntegrity) -> Self {
        let instance = Self {
            algorithm: Some(package_integrity.algorithm.clone()),
            archive_hash: Some(package_integrity.archive_hash.clone()),
        };

        instance
    }

    /**
     * Reset builder
     */
    pub fn reset(&mut self) -> &mut Self {
        self.algorithm = None;
        self.archive_hash = None;

        self
    }

    /**
     * Build package document
     */
    pub fn build(&mut self) -> PackageIntegrityDocument {
        let encoded_archive_hash = hex::encode(
            self.archive_hash
                .clone()
                .expect("Package integrity archive hash must be set"),
        );

        let doc = PackageIntegrityDocument {
            algorithm: self
                .algorithm
                .clone()
                .expect("Package integrity algorithm must be set"),

            archive_hash: encoded_archive_hash,
        };

        self.reset();

        doc
    }
}

impl Default for PackageIntegrityDocumentBuilder {
    fn default() -> Self {
        let instance = Self {
            algorithm: None,
            archive_hash: None,
        };

        instance
    }
}
