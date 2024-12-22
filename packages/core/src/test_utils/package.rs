#[cfg(test)]
pub mod tests {

    use ed25519::signature::{rand_core::OsRng, SignerMut};
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use sha2::{Digest, Sha256};
    use url::Url;

    use crate::packages::{
        package::Package, package_builder::PackageBuilder, package_status::PackageStatus,
        utils::signatures::sign_package,
    };

    pub fn create_package_without_sig(
        maintainer: &VerifyingKey,
    ) -> Result<Package, Box<dyn std::error::Error>> {
        let mut builder = PackageBuilder::default();

        let expected_name = "foo".to_string();
        let expected_version = "1.2.3".to_string();
        let expected_status = PackageStatus::Fine;

        let expected_maintainer = maintainer;

        let expected_archive_url = Url::parse(
            "https://archive.archlinux.org/packages/f/foo/foo-1.2.3-1-x86_64.pkg.tar.zst",
        )?;

        // Pkg integrity
        let expected_integrity_algorithm = "SHA256";

        let mut package_archive_hasher = Sha256::new();
        package_archive_hasher.update("foo");
        let expected_archive_hash = package_archive_hasher.finalize().to_vec();

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
            .build();

        Ok(package)
    }

    pub fn create_package_with_sig() -> Result<Package, Box<dyn std::error::Error>> {
        let mut csprng = OsRng;
        let mut key = SigningKey::generate(&mut csprng);

        let package_without_sig = create_package_without_sig(&key.verifying_key())?;

        let expected_sig = sign_package(&package_without_sig, &mut key);

        let signed_package = PackageBuilder::from_package(&package_without_sig)
            .set_signature(&expected_sig)
            .build();

        Ok(signed_package)
    }
}
