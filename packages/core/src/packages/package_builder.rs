use super::package::Package;

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
     * Package sources, every available sources for package
     */
    sources: Vec<String>,
}

impl PackageBuilder {
    pub fn new() -> Self {
        Self {
            package_name: None,
            package_version: None,
            sources: Vec::new(),
        }
    }

    pub fn reset() -> Self {
        Self {
            package_name: None,
            package_version: None,
            sources: Vec::new(),
        }
    }

    pub fn set_package_name(&mut self, package_name: String) -> &mut Self {
        self.package_name = Some(package_name);
        self
    }

    pub fn set_package_version(&mut self, package_version: String) -> &mut Self {
        self.package_version = Some(package_version);
        self
    }

    pub fn add_source(&mut self, source: String) -> &mut Self {
        self.sources.push(source);
        self
    }

    pub fn build(&self) -> Package {
        if self.sources.is_empty() {
            panic!("At least one package source must be provided")
        }

        Package {
            name: self.package_name.clone().expect("Package name must be set"),
            version: self
                .package_version
                .clone()
                .expect("Package version must be set"),
            sources: self.sources.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::{package::Package, package_builder::PackageBuilder};

    /**
     * It should build package
     */
    #[test]
    fn test_init_config() {
        let package_name_mock = "neofetch";
        let package_version_mock = "1.0.0";
        let package_source_mock = "	https://mirror.xtom.com.hk/archlinux/extra/os/x86_64/neofetch-7.1.0-2-any.pkg.tar.zst";

        let expected_package = Package {
            name: package_name_mock.to_string(),
            version: package_version_mock.to_string(),
            sources: vec![package_name_mock.to_string()],
        };

        let mut builder = PackageBuilder::new();

        let package = builder
            .set_package_name(package_name_mock.to_string())
            .set_package_version(package_version_mock.to_string())
            .add_source(package_source_mock.to_string())
            .build();

        assert_eq!(package.name, package_name_mock);
        assert_eq!(package.version, package_version_mock);
        assert_eq!(package.sources.first().unwrap(), package_source_mock);
    }
}
