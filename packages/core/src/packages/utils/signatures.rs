use ed25519::{signature::SignerMut, Signature};
use ed25519_dalek::SigningKey;
use log::debug;

use crate::packages::package::Package;

/**
 * Sign given package
 */
pub fn sign_package(package: &Package, signing_key: &mut SigningKey) -> Signature {
    let data_integrity_bytes = package.compute_data_integrity();

    let sig = signing_key.sign(&data_integrity_bytes);

    sig
}

/**
 * Verify given package
 */
pub fn verify_package(untrusted_package: &Package) -> Option<&Package> {
    debug!("Verifying {} package signature...", untrusted_package.name);
    let sig = untrusted_package.sig.unwrap();

    let verifying_key = untrusted_package.maintainer;

    let data_integrity = untrusted_package.compute_data_integrity();

    let verification_result = verifying_key.verify_strict(&data_integrity, &sig);

    let verified_package = match verification_result {
        Ok(_) => Some(untrusted_package),
        Err(_) => None,
    };

    debug!(
        "Done verifying {} package signature ! (Passes : {})",
        untrusted_package.name,
        verified_package.is_some()
    );

    verified_package
}

#[cfg(test)]
mod tests {
    use ed25519::signature::{rand_core::OsRng, SignerMut};
    use ed25519_dalek::SigningKey;
    use sha2::{Digest, Sha256};

    use crate::{
        packages::package_builder::PackageBuilder,
        test_utils::package::tests::create_package_with_sig,
    };

    use super::*;

    /**
     * It should verify package
     */
    #[test]
    fn test_verify_package() -> Result<(), Box<dyn std::error::Error>> {
        let package = create_package_with_sig()?;

        let verified_package = verify_package(&package);

        assert_eq!(verified_package.is_some(), true);

        Ok(())
    }

    /**
     * It should not verify package
     */
    #[test]
    fn test_should_not_verify_package() -> Result<(), Box<dyn std::error::Error>> {
        let base_package = create_package_with_sig()?;

        // Sign with another key than the one contained in package's maintainer field
        let mut csprng = OsRng;
        let mut key = SigningKey::generate(&mut csprng);

        let unknown_sig = sign_package(&base_package, &mut key);
        let forged_package = PackageBuilder::from_package(&base_package)
            .set_signature(&unknown_sig)
            .build();

        let verified_package = verify_package(&forged_package);

        assert_eq!(verified_package.is_none(), true);

        Ok(())
    }
}
