use std::{collections::HashMap, path::PathBuf};

use super::package::{Package, PackageBinariesIntegrityMap, PackageIntegrity};

#[derive(Default)]
pub struct PackageBuilder {
    /**
     * Package name
     */
    package_name: Option<String>,
    /**
     * Package version
     */
    package_version: Option<String>,

    /**
     * Package integrity
     */
    package_integrity: Option<PackageIntegrity>,
}

impl PackageBuilder {
    /**
     * Create new package builder instance
     */
    pub fn new() -> Self {
        Self {
            package_name: None,
            package_version: None,
            package_integrity: None,
        }
    }

    /**
     * Reset builder instance
     */
    pub fn reset(&mut self) -> &Self {
        self.package_name = None;
        self.package_version = None;
        self
    }

    /**
     * Parse json and extract package information
     */
    pub fn from_json(raw_package: String) -> Result<Self, serde_json::Error> {
        let package: Package = serde_json::from_str(&raw_package)?;

        let instance = Self {
            package_name: Some(package.name),
            package_version: Some(package.version),
            package_integrity: Some(package.integrity),
        };

        Ok(instance)
    }

    /**
     * Set package name
     */
    pub fn set_package_name(&mut self, package_name: String) -> &mut Self {
        self.package_name = Some(package_name);
        self
    }

    /**
     * Set package version
     */
    pub fn set_package_version(&mut self, package_version: String) -> &mut Self {
        self.package_version = Some(package_version);
        self
    }

    /**
     * Set package integrity data
     */
    pub fn set_package_integrity(
        &mut self,
        integrity_alg: String,
        content_hash: String,
    ) -> &mut Self {
        let package_integrity = PackageIntegrity {
            algorithm: integrity_alg,
            content_hash,
        };

        self.package_integrity = Some(package_integrity);

        self
    }

    /**
     * Build package
     */
    pub fn build(&mut self) -> Package {
        let package = Package {
            name: self.package_name.clone().expect("Package name must be set"),
            version: self
                .package_version
                .clone()
                .expect("Package version must be set"),
            integrity: self
                .package_integrity
                .clone()
                .expect("Package integrity must be set"),
        };

        self.reset();
        package
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::packages::{
        package::{Package, PackageIntegrity},
        package_builder::PackageBuilder,
    };

    /**
     * It should reset package
     */
    #[test]
    fn test_reset_package() {
        let expected_package_name_is_none = true;
        let expected_package_version_is_none = true;

        let package_name_mock = "neofetch";
        let package_version_mock = "7.1.0-2";

        let content_hash_mock = hex::encode("foobar");

        let package_integrity_mock = PackageIntegrity {
            algorithm: "SHA512".to_string(),
            content_hash: content_hash_mock,
        };

        let mut builder = PackageBuilder::new();

        let package = builder
            .set_package_name(package_name_mock.to_string())
            .set_package_version(package_version_mock.to_string())
            .set_package_integrity(
                package_integrity_mock.algorithm,
                package_integrity_mock.content_hash,
            )
            .reset();

        assert_eq!(
            package.package_name.is_none(),
            expected_package_name_is_none
        );

        assert_eq!(
            package.package_version.is_none(),
            expected_package_version_is_none
        );
    }
    /**
     * It should build package
     */
    #[test]
    fn test_build_package() {
        let package_name_mock = "neofetch";
        let package_version_mock = "7.1.0-2";

        let content_hash_mock = hex::encode("foobar");

        let package_integrity_mock = PackageIntegrity {
            algorithm: "SHA512".to_string(),
            content_hash: content_hash_mock,
        };

        let expected_package = Package {
            name: package_name_mock.to_string(),
            version: package_version_mock.to_string(),
            integrity: package_integrity_mock.clone(),
        };

        let mut builder = PackageBuilder::new();

        let package = builder
            .set_package_name(package_name_mock.to_string())
            .set_package_version(package_version_mock.to_string())
            .set_package_integrity(
                package_integrity_mock.algorithm,
                package_integrity_mock.content_hash,
            )
            .build();

        assert_eq!(package.name, expected_package.name);
        assert_eq!(package.version, expected_package.version);
    }
}
