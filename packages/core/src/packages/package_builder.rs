use ed25519::Signature;
use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use rlp::DecoderError;
use url::Url;

use crate::db::documents::package_document::PackageDocument;

use super::{
    package::{Package, PackageStatus},
    package_integrity::PackageIntegrity,
    package_integrity_builder::PackageIntegrityBuilder,
};

#[derive(Default)]
pub struct PackageBuilder {
    /**
     * Package name
     */
    name: Option<String>,
    /**
     * Package version
     */
    version: Option<String>,

    /**
     * Package status
     */
    status: Option<PackageStatus>,

    /**
     * Package maintainer
     */
    maintainer: Option<VerifyingKey>,

    /**
     * Pacakge archive url
     */
    archive_url: Option<Url>,

    /**
     * Package integrity
     */
    integrity: Option<PackageIntegrity>,

    /**
     * Package signature
     */
    sig: Option<Signature>,
}

impl PackageBuilder {
    /**
     * Build from document
     */
    pub fn from_document(document: &PackageDocument) -> PackageBuilder {
        // Package status
        let package_status_integer = document.status as u8;
        let package_status = PackageStatus::try_from(package_status_integer)
            .expect("Could not convert package status from integer to enum");

        // Package maintainer
        let package_maintainer_decoded =
            hex::decode(document.maintainer.clone()).expect("Could not decode package maintainer");

        let mut package_maintainer_buf: [u8; PUBLIC_KEY_LENGTH] = [0; PUBLIC_KEY_LENGTH];

        package_maintainer_buf.copy_from_slice(package_maintainer_decoded.as_slice());

        let package_maintainer = VerifyingKey::from_bytes(&package_maintainer_buf)
            .expect("Could not build key from decoded maintainer key");

        // Package archive url
        let archive_url = Url::parse(&document.archive_url.as_str()).unwrap();

        // Package integrity

        let package_integrity = PackageIntegrityBuilder::from_document(&document.integrity).build();

        // Package signature

        let mut package_signature_buf: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];

        let decoded_sig = hex::decode(&document.sig).unwrap();
        package_signature_buf.copy_from_slice(&decoded_sig);

        let package_signature = Signature::from_bytes(&package_signature_buf);

        Self {
            name: Some(document.name.clone()),
            version: Some(document.version.clone()),
            status: Some(package_status),
            maintainer: Some(package_maintainer),
            archive_url: Some(archive_url),
            integrity: Some(package_integrity),
            sig: Some(package_signature),
        }
    }

    /**
     * Create new package builder instance
     */
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            status: None,
            maintainer: None,
            archive_url: None,
            integrity: None,
            sig: None,
        }
    }

    /**
     * Reset builder instance
     */
    pub fn reset(&mut self) -> &Self {
        self.name = None;
        self.version = None;
        self.status = None;
        self.maintainer = None;
        self.archive_url = None;
        self.integrity = None;
        self.sig = None;
        self
    }

    /**
     * Build from other package
     */
    pub fn from_package(package: &Package) -> Self {
        let instance = Self {
            name: Some(package.name.clone()),
            version: Some(package.version.clone()),
            status: Some(package.status.clone()),
            maintainer: Some(package.maintainer),
            archive_url: Some(package.archive_url.clone()),
            integrity: Some(package.integrity.clone()),
            sig: package.sig,
        };

        instance
    }

    /**
     * Parse rpl and extract package information
     */
    pub fn from_rpl(raw_package: &[u8]) -> Result<Self, DecoderError> {
        let package: Package = rlp::decode(&raw_package)?;

        let instance = Self {
            name: Some(package.name),
            version: Some(package.version),
            status: Some(package.status),
            maintainer: Some(package.maintainer),
            archive_url: Some(package.archive_url),
            integrity: Some(package.integrity),
            sig: package.sig,
        };

        Ok(instance)
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
        self.status = Some(status.clone());
        self
    }

    /**
     * Set package maitainer
     */
    pub fn set_maintainer(&mut self, maintainer: &VerifyingKey) -> &mut Self {
        self.maintainer = Some(maintainer.clone());
        self
    }

    /**
     * Set archive url
     */
    pub fn set_archive_url(&mut self, archive_url: &Url) -> &mut Self {
        self.archive_url = Some(archive_url.clone());
        self
    }

    /**
     * Set package integrity data
     */
    pub fn set_integrity(&mut self, integrity_alg: &String, archive_hash: &[u8]) -> &mut Self {
        let integrity = PackageIntegrity {
            algorithm: integrity_alg.clone(),
            archive_hash: Vec::from(archive_hash),
        };

        self.integrity = Some(integrity);

        self
    }

    /**
     * Set package signature
     */
    pub fn set_signature(&mut self, sig: &Signature) -> &mut Self {
        self.sig = Some(sig.clone());
        self
    }

    /**
     * Build package
     */
    pub fn build(&mut self) -> Package {
        let package = Package {
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

            sig: self.sig.clone(),
        };

        self.reset();

        package
    }
}

#[cfg(test)]
mod tests {

    use ed25519::signature::{rand_core::OsRng, SignerMut};
    use ed25519_dalek::SigningKey;
    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn test_package_build() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = PackageBuilder::new();

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

        let package = builder
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

        assert_eq!(package.name, expected_name);
        assert_eq!(package.version, expected_version);
        assert_eq!(package.status, expected_status);
        assert_eq!(package.maintainer, expected_maintainer);
        assert_eq!(package.archive_url, expected_archive_url);
        assert_eq!(package.sig.unwrap(), expected_sig);

        Ok(())
    }

    #[test]
    fn test_package_build_from_package() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = PackageBuilder::new();

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

        let package = builder
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

        let copied_package = PackageBuilder::from_package(&package).build();

        assert_eq!(copied_package.name, package.name);
        assert_eq!(copied_package.version, package.version);
        assert_eq!(copied_package.status, package.status);
        assert_eq!(copied_package.maintainer, package.maintainer);
        assert_eq!(copied_package.archive_url, package.archive_url);
        assert_eq!(copied_package.sig.unwrap(), package.sig.unwrap());

        Ok(())
    }

    #[test]
    fn test_package_reset() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = PackageBuilder::new();

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

        let package = builder
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

        assert_eq!(builder.name, None);
        assert_eq!(builder.version, None);
        assert_eq!(builder.status, None);
        assert_eq!(builder.maintainer, None);
        assert_eq!(builder.archive_url, None);
        assert_eq!(builder.sig, None);

        Ok(())
    }
}
