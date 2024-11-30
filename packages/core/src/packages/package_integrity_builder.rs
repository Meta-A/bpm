use rlp::DecoderError;

use crate::db::documents::package_integrity_document::PackageIntegrityDocument;

use super::package_integrity::PackageIntegrity;

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
     * Parse rlp and extract package integer information
     */
    pub fn from_rlp(raw_package_integrity: &[u8]) -> Result<Self, DecoderError> {
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

impl Default for PackageIntegrityBuilder {
    fn default() -> Self {
        Self {
            algorithm: None,
            archive_hash: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use crate::db::documents::package_integrity_document_builder::PackageIntegrityDocumentBuilder;

    use super::PackageIntegrityBuilder;

    use super::*;

    /**
     * It should build package integrity
     */
    #[test]
    fn test_package_integrity_build() {
        let expected_algorithm = "SHA256".to_string();

        let mut package_archive_hasher = Sha256::new();
        package_archive_hasher.update("foo");
        let expected_archive_hash = package_archive_hasher.finalize().to_vec();

        let package_integrity = PackageIntegrityBuilder::default()
            .set_algorithm(&expected_algorithm)
            .set_archive_hash(&expected_archive_hash)
            .build();

        assert_eq!(package_integrity.algorithm, expected_algorithm);
        assert_eq!(package_integrity.archive_hash, expected_archive_hash);
    }

    /**
     * It should reset builder
     */
    #[test]
    fn test_package_integrity_reset() {
        let expected_algorithm = "SHA256".to_string();

        let mut builder = PackageIntegrityBuilder::default();
        let package_integrity = builder.set_algorithm(&expected_algorithm).reset();

        assert_eq!(package_integrity.algorithm, None);
        assert_eq!(package_integrity.archive_hash, None);
    }

    /**
     * It should build from other package integrity
     */
    #[test]
    fn test_package_integrity_build_from_package_integrity() {
        let expected_algorithm = "SHA256".to_string();

        let mut package_archive_hasher = Sha256::new();
        package_archive_hasher.update("foo");
        let expected_archive_hash = package_archive_hasher.finalize().to_vec();

        let package_integrity = PackageIntegrityBuilder::default()
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

    /**
     * It should build package integrity from document
     */
    #[test]
    fn test_package_integrity_build_from_package_integrity_doc() {
        let mut hasher = Sha256::new();

        hasher.update("foo");

        let expected_algorithm = "SHA256";
        let expected_archive_hash = hasher.finalize().to_vec();

        let mut doc_builder = PackageIntegrityDocumentBuilder::default();
        let doc = doc_builder
            .set_algorithm(&expected_algorithm.to_string())
            .set_archive_hash(&expected_archive_hash)
            .build();

        let package_integrity = PackageIntegrityBuilder::from_document(&doc).build();

        assert_eq!(doc.algorithm, package_integrity.algorithm);
        assert_eq!(
            doc.archive_hash,
            hex::encode(package_integrity.archive_hash)
        );
    }

    #[test]
    fn test_package_integrity_build_from_rlp() -> Result<(), Box<dyn std::error::Error>> {
        let expected_algorithm = "SHA256".to_string();

        let mut package_archive_hasher = Sha256::new();
        package_archive_hasher.update("foo");
        let expected_archive_hash = package_archive_hasher.finalize().to_vec();

        let package_integrity = PackageIntegrityBuilder::default()
            .set_algorithm(&expected_algorithm)
            .set_archive_hash(&expected_archive_hash)
            .build();

        let encoded_package_integrity = rlp::encode(&package_integrity);

        let decoded_package_integrity =
            PackageIntegrityBuilder::from_rlp(&encoded_package_integrity)?.build();

        assert_eq!(
            decoded_package_integrity.algorithm,
            package_integrity.algorithm
        );
        assert_eq!(
            decoded_package_integrity.archive_hash,
            package_integrity.archive_hash
        );

        Ok(())
    }
}
