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
     * Set algorithm
     */
    pub fn set_algorithm(&mut self, algorithm: &String) -> &mut Self {
        self.algorithm = Some(algorithm.clone());

        self
    }

    /**
     * Set archive hash
     */
    pub fn set_archive_hash(&mut self, archive_hash: &Vec<u8>) -> &mut Self {
        self.archive_hash = Some(archive_hash.clone());

        self
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

mod tests {
    use sha2::{Digest, Sha256};

    use super::PackageIntegrityBuilder;

    use super::*;

    #[test]
    fn test_package_integrity_build() {
        let expected_algorithm = "SHA256".to_string();

        let mut package_archive_hasher = Sha256::new();
        package_archive_hasher.update("foo");
        let expected_archive_hash = package_archive_hasher.finalize().to_vec();

        let package_integrity = PackageIntegrityBuilder::new()
            .set_algorithm(&expected_algorithm)
            .set_archive_hash(&expected_archive_hash)
            .build();

        assert_eq!(package_integrity.algorithm, expected_algorithm);
        assert_eq!(package_integrity.archive_hash, expected_archive_hash);
    }

    #[test]
    fn test_package_integrity_reset() {
        let expected_algorithm = "SHA256".to_string();

        let mut builder = PackageIntegrityBuilder::new();
        let package_integrity = builder.set_algorithm(&expected_algorithm).reset();

        assert_eq!(package_integrity.algorithm, None);
    }

    #[test]
    fn test_package_integrity_build_from_package_integrity() {
        let expected_algorithm = "SHA256".to_string();

        let mut package_archive_hasher = Sha256::new();
        package_archive_hasher.update("foo");
        let expected_archive_hash = package_archive_hasher.finalize().to_vec();

        let package_integrity = PackageIntegrityBuilder::new()
            .set_algorithm(&expected_algorithm)
            .set_archive_hash(&expected_archive_hash)
            .build();

        assert_eq!(package_integrity.algorithm, expected_algorithm);
        assert_eq!(package_integrity.archive_hash, expected_archive_hash);

        let copied_package_integrity =
            PackageIntegrityBuilder::from_package_integrity(&package_integrity).build();
        assert_eq!(
            copied_package_integrity.algorithm,
            package_integrity.algorithm
        );
        assert_eq!(
            copied_package_integrity.archive_hash,
            package_integrity.archive_hash
        );
    }
}
