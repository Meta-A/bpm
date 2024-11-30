use polodb_core::bson::{Bson, Document};

use super::package_integrity_document::PackageIntegrityDocument;

/**
 * Represent how package is stored in DB
 */
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PackageDocument {
    pub name: String,
    pub version: String,
    pub status: i32,
    pub maintainer: String,
    pub archive_url: String,
    pub integrity: PackageIntegrityDocument,
    pub sig: String,
    pub blockchain_label: String,
}

impl Into<Bson> for &PackageDocument {
    fn into(self) -> Bson {
        let mut doc = Document::new();

        doc.insert("name", &self.name);

        doc.insert("version", &self.version);

        doc.insert("status", &self.status);

        doc.insert("maintainer", &self.maintainer);

        doc.insert("archive_url", &self.archive_url);

        let integrity: Bson = (&self.integrity).into();
        doc.insert("integrity", integrity);

        doc.insert("sig", &self.sig);

        doc.insert("blockchain_label", &self.blockchain_label);

        Bson::Document(doc)
    }
}

#[cfg(test)]
mod tests {
    use ed25519::signature::{rand_core::OsRng, SignerMut};
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use sha2::{Digest, Sha256};

    use crate::{
        db::documents::package_integrity_document_builder::PackageIntegrityDocumentBuilder,
        packages::{
            package_integrity_builder::PackageIntegrityBuilder, package_status::PackageStatus,
        },
    };

    use super::*;

    #[test]
    fn test_should_convert_to_bson() {
        let expected_name = "foo";
        let expected_version = "1.2.3";
        let expected_status = i32::from(PackageStatus::Fine as u8);

        let mut csprng = OsRng;
        let mut key = SigningKey::generate(&mut csprng);

        let maintainer = key.verifying_key();

        let archive_url =
            "https://archive.archlinux.org/packages/f/foo/foo-1.2.3-1-x86_64.pkg.tar.zst";

        // Package integrity

        let mut package_integrity_document_builder = PackageIntegrityDocumentBuilder::default();
        let mut hasher = Sha256::new();

        hasher.update("foo");

        let expected_integrity_algorithm = "SHA256";
        let expected_integrity_archive_hash = hasher.finalize().to_vec();

        let package_integrity = package_integrity_document_builder
            .set_algorithm(&expected_integrity_algorithm.to_string())
            .set_archive_hash(&expected_integrity_archive_hash)
            .build();

        let package_info_hash_data = format!(
            "{expected_name}{expected_version}{}{}{archive_url}{}{}",
            expected_status,
            hex::encode(maintainer),
            package_integrity.algorithm,
            hex::encode(package_integrity.archive_hash.clone())
        );

        let mut hasher = Sha256::new();
        hasher.update(package_info_hash_data);
        let package_data_hash = hasher.finalize().to_vec();

        let package_info_hash = hex::encode(package_data_hash);

        let package_sig = key.sign(&package_info_hash.as_bytes()).to_bytes();

        let blockchain_label = "hedera";

        let package_document = PackageDocument {
            name: expected_name.to_string().clone(),
            version: expected_version.to_string().clone(),
            status: expected_status.clone(),
            maintainer: hex::encode(maintainer),
            archive_url: archive_url.to_string(),
            integrity: package_integrity.clone(),
            sig: hex::encode(package_sig).clone(),
            blockchain_label: blockchain_label.to_string(),
        };

        let bson_repr: Bson = (&package_document).into();
        let bson_doc = bson_repr.as_document().unwrap();

        assert_eq!(package_document.name, bson_doc.get_str("name").unwrap());

        assert_eq!(
            package_document.version,
            bson_doc.get_str("version").unwrap()
        );
        assert_eq!(package_document.status, bson_doc.get_i32("status").unwrap());

        assert_eq!(
            package_document.maintainer,
            bson_doc.get_str("maintainer").unwrap()
        );

        assert_eq!(
            package_document.archive_url,
            bson_doc.get_str("archive_url").unwrap()
        );
    }
}
