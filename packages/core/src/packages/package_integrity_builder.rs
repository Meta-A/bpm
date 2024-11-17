use rlp::DecoderError;

use crate::db::documents::package_integrity_document::PackageIntegrityDocument;

use super::package_integrity::PackageIntegrity;

#[derive(Default)]
pub struct PackageIntegrityBuilder {
    algorithm: Option<String>,
    archive_hash: Option<Vec<u8>>,
}

impl PackageIntegrityBuilder {
    /**
     * Build from document
     */
    pub fn from_document(document: &PackageIntegrityDocument) -> PackageIntegrityBuilder {
        let decoded_archive_hash = hex::decode(&document.archive_hash).unwrap();
        Self {
            algorithm: Some(document.algorithm.clone()),
            archive_hash: Some(decoded_archive_hash),
        }
    }

    /**
     * Create new package builder instance
     */
    pub fn new() -> Self {
        Self {
            algorithm: None,
            archive_hash: None,
        }
    }

    /**
     * Reset builder instance
     */
    pub fn reset(&mut self) -> &Self {
        self.algorithm = None;
        self.archive_hash = None;
        self
    }

    /**
     * Build from other package integrity
     */
    pub fn from_package_integrity(package_integrity: &PackageIntegrity) -> Self {
        let instance = Self {
            algorithm: Some(package_integrity.algorithm.clone()),
            archive_hash: Some(package_integrity.archive_hash.clone()),
        };

        instance
    }

    /**
     * Parse rpl and extract package integer information
     */
    pub fn from_rpl(raw_package_integrity: &[u8]) -> Result<Self, DecoderError> {
        let package_integrity: PackageIntegrity = rlp::decode(&raw_package_integrity)?;

        let instance = Self {
            algorithm: Some(package_integrity.algorithm),
            archive_hash: Some(package_integrity.archive_hash),
        };

        Ok(instance)
    }

    /**
     * Build package integrity
     */
    pub fn build(&mut self) -> PackageIntegrity {
        let package_integrity = PackageIntegrity {
            algorithm: self
                .algorithm
                .clone()
                .expect("Package algorithm must be set"),
            archive_hash: self
                .archive_hash
                .clone()
                .expect("Package archive hash must be set"),
        };

        self.reset();

        package_integrity
    }
}
