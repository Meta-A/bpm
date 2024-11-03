use ed25519::Signature;
use ed25519_dalek::VerifyingKey;
use rlp::DecoderError;

use super::package::{Package, PackageIntegrity, PackageStatus};

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
     * Create new package builder instance
     */
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            status: None,
            maintainer: None,
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
            integrity: Some(package.integrity),
            sig: package.sig,
        };

        Ok(instance)
    }

    /**
     * Set package name
     */
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    /**
     * Set package version
     */
    pub fn set_version(&mut self, version: String) -> &mut Self {
        self.version = Some(version);
        self
    }

    /**
     * Set package status
     */
    pub fn set_status(&mut self, status: PackageStatus) -> &mut Self {
        self.status = Some(status);
        self
    }

    /**
     * Set package maitainer
     */
    pub fn set_maintainer(&mut self, maintainer: VerifyingKey) -> &mut Self {
        self.maintainer = Some(maintainer);
        self
    }

    /**
     * Set package integrity data
     */
    pub fn set_integrity(&mut self, integrity_alg: String, archive_hash: &[u8]) -> &mut Self {
        let integrity = PackageIntegrity {
            algorithm: integrity_alg,
            archive_hash: Vec::from(archive_hash),
        };

        self.integrity = Some(integrity);

        self
    }

    /**
     * Set package signature
     */
    pub fn set_signature(&mut self, sig: Signature) -> &mut Self {
        self.sig = Some(sig);
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

    use crate::packages::{
        package::{Package, PackageIntegrity},
        package_builder::PackageBuilder,
    };

    // * It should reset package
    //#[test]
    //fn test_reset_package() {
    //    let expected_name_is_none = true;
    //    let expected_version_is_none = true;
    //
    //    let name_mock = "neofetch";
    //    let version_mock = "7.1.0-2";
    //
    //    let archive_hash_mock = hex::encode("foobar");
    //
    //    let integrity_mock = PackageIntegrity {
    //        algorithm: "SHA256".to_string(),
    //        archive_hash: archive_hash_mock,
    //    };
    //
    //    let mut builder = PackageBuilder::new();
    //
    //    let package = builder
    //        .set_name(name_mock.to_string())
    //        .set_version(version_mock.to_string())
    //        .set_integrity(
    //            integrity_mock.algorithm,
    //            integrity_mock.archive_hash,
    //        )
    //        .reset();
    //
    //    assert_eq!(
    //        package.name.is_none(),
    //        expected_name_is_none
    //    );
    //
    //    assert_eq!(
    //        package.version.is_none(),
    //        expected_version_is_none
    //    );
    //}
    // * It should build package
    //#[test]
    //fn test_build_package() {
    //    let name_mock = "neofetch";
    //    let version_mock = "7.1.0-2";
    //
    //    let archive_hash_mock = hex::encode("foobar");
    //
    //    let integrity_mock = PackageIntegrity {
    //        algorithm: "SHA256".to_string(),
    //        archive_hash: archive_hash_mock,
    //    };
    //
    //    let expected_package = Package {
    //        name: name_mock.to_string(),
    //        version: version_mock.to_string(),
    //        integrity: integrity_mock.clone(),
    //    };
    //
    //    let mut builder = PackageBuilder::new();
    //
    //    let package = builder
    //        .set_name(name_mock.to_string())
    //        .set_version(version_mock.to_string())
    //        .set_integrity(
    //            integrity_mock.algorithm,
    //            integrity_mock.archive_hash,
    //        )
    //        .build();
    //
    //    assert_eq!(package.name, expected_package.name);
    //    assert_eq!(package.version, expected_package.version);
    //}
}
