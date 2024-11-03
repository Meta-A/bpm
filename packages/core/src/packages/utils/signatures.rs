use ed25519::{signature::SignerMut, Signature};
use ed25519_dalek::SigningKey;
use log::debug;

use crate::packages::package::Package;

pub fn sign_package(package: &Package, signing_key: &mut SigningKey) -> Signature {
    let data_integrity_bytes = package.compute_data_integrity();

    let sig = signing_key.sign(&data_integrity_bytes);

    sig
}

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
