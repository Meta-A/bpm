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

#[cfg(test)]
mod tests {

    use sha2::{Digest, Sha256};

    use crate::packages::package_integrity;

    use super::*;

    #[test]
    fn test_package_integrity_build() {
        let mut builder = PackageIntegrityDocumentBuilder::default();

        let mut hasher = Sha256::new();

        hasher.update("foo");

        let expected_algorithm = "SHA256";
        let expected_archive_hash = hasher.finalize().to_vec();

        let doc = builder
            .set_algorithm(&expected_algorithm.to_string())
            .set_archive_hash(&expected_archive_hash)
            .build();

        assert_eq!(doc.algorithm, expected_algorithm);
        assert_eq!(doc.archive_hash, hex::encode(&expected_archive_hash));
    }

    #[test]
    fn test_package_integrity_reset() {
        let mut builder = PackageIntegrityDocumentBuilder::default();

        let mut hasher = Sha256::new();

        hasher.update("foo");

        let expected_algorithm = "SHA256";
        let expected_archive_hash = hasher.finalize().to_vec();

        let doc = builder
            .set_algorithm(&expected_algorithm.to_string())
            .set_archive_hash(&expected_archive_hash)
            .reset();

        assert_eq!(doc.algorithm, None);
        assert_eq!(doc.archive_hash, None);
    }

    #[test]
    fn test_package_integrity_build_from_package_integrity() {
        let mut hasher = Sha256::new();

        hasher.update("foo");

        let expected_algorithm = "SHA256";
        let expected_archive_hash = hasher.finalize().to_vec();

        let package_integrity: PackageIntegrity = PackageIntegrity {
            algorithm: expected_algorithm.to_string(),
            archive_hash: expected_archive_hash.clone(),
        };

        let mut builder =
            PackageIntegrityDocumentBuilder::from_package_integrity(&package_integrity);

        let doc = builder.build();
        assert_eq!(doc.algorithm, expected_algorithm);
        assert_eq!(doc.archive_hash, hex::encode(&expected_archive_hash));
    }
}
