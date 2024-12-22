use ed25519::Signature;
use ed25519_dalek::VerifyingKey;
use url::Url;

use crate::{
    blockchains::blockchain::BlockchainClient,
    packages::{package::Package, package_status::PackageStatus},
};

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
     * Set package name
     */
    pub fn set_name(&mut self, name: &String) -> &mut Self {
        self.name = Some(name.clone());
        self
    }

    /**
     * Set package version
     */
    pub fn set_version(&mut self, version: &String) -> &mut Self {
        self.version = Some(version.clone());
        self
    }

    /**
     * Set package status
     */
    pub fn set_status(&mut self, status: &PackageStatus) -> &mut Self {
        let status: u8 = status.clone() as u8;
        self.status = Some(i32::from(status));
        self
    }

    /**
     * Set maintainer
     */
    pub fn set_maintainer(&mut self, maintainer: &VerifyingKey) -> &mut Self {
        let encoded_maintainer = hex::encode(maintainer.to_bytes());

        self.maintainer = Some(encoded_maintainer);
        self
    }

    /**
     * Set package archive URL
     */
    pub fn set_archive_url(&mut self, archive_url: &Url) -> &mut Self {
        self.archive_url = Some(archive_url.to_string());

        self
    }

    /**
     * Set package integrity
     */
    pub fn set_integrity(&mut self, integrity: &PackageIntegrityDocument) -> &mut Self {
        self.integrity = Some(integrity.clone());
        self
    }

    /**
     * Set package signature
     */
    pub fn set_signature(&mut self, signature: &Signature) -> &mut Self {
        self.sig = Some(signature.to_vec());
        self
    }

    /**
     * Set blockchain label
     */
    pub fn set_blockchain(&mut self, blockchain_client: &Box<dyn BlockchainClient>) -> &mut Self {
        self.blockchain_label = Some(blockchain_client.get_label().clone());
        self
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
     * Build from document
     */
    pub fn from_document(doc: &PackageDocument) -> Self {
        let sig = hex::decode(doc.sig.clone()).unwrap();
        let instance = Self {
            name: Some(doc.name.clone()),
            version: Some(doc.version.clone()),
            status: Some(doc.status.clone()),
            maintainer: Some(doc.maintainer.clone()),
            archive_url: Some(doc.archive_url.clone()),
            integrity: Some(doc.integrity.clone()),
            sig: Some(sig),
            blockchain_label: Some(doc.blockchain_label.clone()),
        };

        instance
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

#[cfg(test)]
mod tests {
    use ed25519::signature::{rand_core::OsRng, SignerMut};
    use ed25519_dalek::SigningKey;
    use sha2::{Digest, Sha256};

    use crate::{
        blockchains::hedera::blockchain_client::HederaBlockchain,
        packages::{package_builder::PackageBuilder, package_status::PackageStatus},
    };

    use super::*;

    #[tokio::test]
    async fn test_package_document_build() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = PackageDocumentBuilder::default();

        let expected_name = "foo".to_string();
        let expected_version = "1.2.3".to_string();
        let expected_status = PackageStatus::Fine;

        let mut csprng = OsRng;
        let mut key = SigningKey::generate(&mut csprng);

        let expected_maintainer = key.verifying_key();

        let expected_archive_url = Url::parse(
            "https://archive.archlinux.org/packages/f/foo/foo-1.2.3-1-x86_64.pkg.tar.zst",
        )?;

        // Pkg integrity
        let expected_integrity_algorithm = "SHA256".to_string();

        let mut package_archive_hasher = Sha256::new();
        package_archive_hasher.update("foo");

        let expected_archive_hash = package_archive_hasher.finalize().to_vec();

        let package_integrity_document = PackageIntegrityDocumentBuilder::default()
            .set_algorithm(&expected_integrity_algorithm)
            .set_archive_hash(&expected_archive_hash)
            .build();

        // Pkg sig
        let package_info_hash_data = format!(
            "{expected_name}{expected_version}{}{}{expected_archive_url}{}{}",
            expected_status,
            hex::encode(expected_maintainer),
            expected_integrity_algorithm,
            hex::encode(expected_archive_hash.clone())
        );

        let mut package_sig_hasher = Sha256::new();

        package_sig_hasher.update(package_info_hash_data);

        let package_data_hash = package_sig_hasher.finalize();

        let expected_sig = key.sign(&package_data_hash);

        // Pkg related blockchain

        let blockhain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        let expected_blockchain_label = blockhain_client.get_label();

        let package_doc = builder
            .set_name(&expected_name)
            .set_version(&expected_version)
            .set_status(&expected_status)
            .set_maintainer(&expected_maintainer)
            .set_archive_url(&expected_archive_url)
            .set_blockchain(&blockhain_client)
            .set_integrity(&package_integrity_document)
            .set_signature(&expected_sig)
            .build();

        assert_eq!(package_doc.name, expected_name);
        assert_eq!(package_doc.version, expected_version);
        assert_eq!(package_doc.status, i32::from(expected_status as u8));
        assert_eq!(package_doc.blockchain_label, expected_blockchain_label);
        assert_eq!(
            package_doc.maintainer,
            hex::encode(expected_maintainer.to_bytes())
        );
        assert_eq!(package_doc.archive_url, expected_archive_url.to_string());
        assert_eq!(package_doc.sig, hex::encode(expected_sig.to_vec()));

        Ok(())
    }

    #[tokio::test]
    async fn test_package_document_build_from_package() -> Result<(), Box<dyn std::error::Error>> {
        let expected_name = "foo".to_string();
        let expected_version = "1.2.3".to_string();
        let expected_status = PackageStatus::Fine;

        let mut csprng = OsRng;
        let mut key = SigningKey::generate(&mut csprng);

        let expected_maintainer = key.verifying_key();

        let expected_archive_url = Url::parse(
            "https://archive.archlinux.org/packages/f/foo/foo-1.2.3-1-x86_64.pkg.tar.zst",
        )?;

        // Pkg integrity
        let expected_integrity_algorithm = "SHA256";

        let mut package_archive_hasher = Sha256::new();
        package_archive_hasher.update("foo");
        let expected_archive_hash = package_archive_hasher.finalize().to_vec();

        // Pkg sig
        let package_info_hash_data = format!(
            "{expected_name}{expected_version}{}{}{expected_archive_url}{}{}",
            expected_status,
            hex::encode(expected_maintainer),
            expected_integrity_algorithm,
            hex::encode(expected_archive_hash.clone())
        );

        let mut package_sig_hasher = Sha256::new();

        package_sig_hasher.update(package_info_hash_data);

        let package_data_hash = package_sig_hasher.finalize();

        let expected_sig = key.sign(&package_data_hash);

        // Package builder

        let package = PackageBuilder::default()
            .set_name(&expected_name)
            .set_version(&expected_version)
            .set_status(&expected_status)
            .set_maintainer(&expected_maintainer)
            .set_archive_url(&expected_archive_url)
            .set_integrity(
                &expected_integrity_algorithm.to_string(),
                &expected_archive_hash,
            )
            .set_signature(&expected_sig)
            .build();

        let blockhain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::from("4991716"));

        let expected_blockchain_label = blockhain_client.get_label();

        let package_doc = PackageDocumentBuilder::from_package(&package, &blockhain_client).build();

        assert_eq!(package_doc.name, expected_name);
        assert_eq!(package_doc.version, expected_version);
        assert_eq!(package_doc.status, i32::from(expected_status as u8));
        assert_eq!(package_doc.blockchain_label, expected_blockchain_label);
        assert_eq!(
            package_doc.maintainer,
            hex::encode(expected_maintainer.to_bytes())
        );
        assert_eq!(package_doc.archive_url, expected_archive_url.to_string());
        assert_eq!(package_doc.sig, hex::encode(expected_sig.to_vec()));

        Ok(())
    }
}
