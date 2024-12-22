use crate::packages::package_integrity::PackageIntegrity;

use super::package_builder::PackageBuilder;
use super::package_status::PackageStatus;
use core::fmt;
use ed25519::Signature;
use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use rlp::{Decodable, DecoderError, Encodable, RlpStream};
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{
    ser::{Error, SerializeStruct},
    Deserialize, Serialize,
};
use sha2::{Digest, Sha256};
use url::Url;

pub const DEFAULT_PACKAGE_STATUS: PackageStatus = PackageStatus::Fine;

/**
 * Package
 */
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub status: PackageStatus,
    pub maintainer: VerifyingKey, // Maintainer is identified by its public key
    pub archive_url: Url,         // TODO: Convert to list
    pub integrity: PackageIntegrity,
    pub sig: Option<Signature>,
}

impl Package {
    /**
     * Return data bytes, mostly used to compute sig hash
     */
    pub fn compute_data_integrity(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();

        let data = self.get_rlp_data_stream().as_raw().to_vec();

        hasher.update(data);

        let hash = hasher.finalize();

        hash.to_vec()
    }

    /**
     * Create RLP stream that only contains data
     */
    fn get_rlp_data_stream(&self) -> RlpStream {
        let encoded_package_integrity = rlp::encode(&self.integrity);
        let mut stream = rlp::RlpStream::new();

        let encoded_status = self.status.clone() as u8;
        stream
            // Package name
            .append(&self.name)
            // Package version
            .append(&self.version)
            // Package status
            .append(&encoded_status)
            // Package maintainer
            .append(&self.maintainer.to_bytes().as_slice())
            // Package archive urls
            .append(&self.archive_url.as_str())
            // Package integrity
            .append_list(&encoded_package_integrity);

        stream
    }

    pub fn builder() -> PackageBuilder {
        PackageBuilder::default()
    }
}

// Serde encoding / decoding
impl Serialize for Package {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Package", 5)?;
        state.serialize_field("name", &self.name)?;

        state.serialize_field("version", &self.version)?;

        let encoded_status = self.status.clone() as u8;
        state.serialize_field("status", &encoded_status)?;

        state.serialize_field("maintainer", &self.maintainer.to_bytes())?;

        state.serialize_field("archive_url", &self.archive_url.to_string())?;

        state.serialize_field("integrity", &self.integrity)?;

        let sig = match self.sig {
            Some(v) => v,
            None => {
                return Err(S::Error::custom(
                    "Signature must be attached to package when serializing it",
                ))
            }
        };

        state.serialize_field("sig", &sig.to_bytes().as_slice())?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for Package {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Name,
            Version,
            Status,
            Maintainer,
            Archive_Url,
            Integrity,
            Sig,
        }
        struct PackageVisitor;

        impl<'de> Visitor<'de> for PackageVisitor {
            type Value = Package;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Package")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Package, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut version = None;
                let mut status = None;
                let mut maintainer = None;
                let mut archive_url = None;
                let mut integrity = None;
                let mut sig = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value()?);
                        }
                        Field::Status => {
                            if status.is_some() {
                                return Err(de::Error::duplicate_field("status"));
                            }

                            let raw_status: u8 = map.next_value()?;

                            status = Some(
                                PackageStatus::try_from(raw_status)
                                    .map_err(|e| de::Error::custom(e))?,
                            );
                        }
                        Field::Maintainer => {
                            if maintainer.is_some() {
                                return Err(de::Error::duplicate_field("maintainer"));
                            }

                            let mut maintainer_raw_key_buf: [u8; PUBLIC_KEY_LENGTH] =
                                [0; PUBLIC_KEY_LENGTH];

                            let maintainer_key_bytes: Vec<u8> = map.next_value()?;

                            maintainer_raw_key_buf.copy_from_slice(&maintainer_key_bytes);

                            maintainer = Some(
                                VerifyingKey::from_bytes(&maintainer_raw_key_buf)
                                    .map_err(|_| DecoderError::RlpExpectedToBeData)
                                    .unwrap(),
                            );
                        }

                        Field::Archive_Url => {
                            if archive_url.is_some() {
                                return Err(de::Error::duplicate_field("archive_url"));
                            }

                            let raw_url = map.next_value()?;
                            archive_url = Some(Url::parse(raw_url).unwrap());
                        }

                        Field::Integrity => {
                            if integrity.is_some() {
                                return Err(de::Error::duplicate_field("integrity"));
                            }
                            integrity = Some(map.next_value()?);
                        }

                        Field::Sig => {
                            if sig.is_some() {
                                return Err(de::Error::duplicate_field("maintainer"));
                            }

                            let mut sig_buf: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];

                            let sig_bytes: Vec<u8> = map.next_value()?;

                            sig_buf.copy_from_slice(&sig_bytes);

                            sig = Some(Some(Signature::from_bytes(&sig_buf)));
                        }
                    }
                }
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let status = status.ok_or_else(|| de::Error::missing_field("status"))?;
                let maintainer =
                    maintainer.ok_or_else(|| de::Error::missing_field("maintainer"))?;

                let archive_url =
                    archive_url.ok_or_else(|| de::Error::missing_field("archive_url"))?;

                let integrity = integrity.ok_or_else(|| de::Error::missing_field("integrity"))?;
                let sig = sig.ok_or_else(|| de::Error::missing_field("sig"))?;

                let package = Package {
                    name,
                    version,
                    status,
                    maintainer,
                    archive_url,
                    integrity,
                    sig,
                };
                Ok(package)
            }
        }

        const FIELDS: &[&str] = &["secs", "nanos"];
        deserializer.deserialize_struct("Package", FIELDS, PackageVisitor)
    }
}

// RLP encoding / decoding

impl Encodable for Package {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let sig = match self.sig {
            Some(v) => v,
            None => panic!("Signature must be attached to package when encoding it"),
        };

        let data_stream = self.get_rlp_data_stream();
        s.begin_unbounded_list();
        // Data
        s.append_raw(data_stream.as_raw(), data_stream.len());
        // Signature
        s.append(&sig.to_bytes().as_slice());
        s.finalize_unbounded_list();
    }
}

impl Decodable for Package {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        // Parse name
        let name: String = rlp.val_at(0)?;

        // Parse version
        let version: String = rlp.val_at(1)?;

        // Parse status
        let raw_status: u8 = rlp.val_at(2)?;

        let status = PackageStatus::try_from(raw_status)
            .map_err(|_| rlp::DecoderError::RlpInconsistentLengthAndData)?;

        // Parse maintainer verifying key
        let mut maintainer_raw_key_buf: [u8; PUBLIC_KEY_LENGTH] = [0; PUBLIC_KEY_LENGTH];

        let maintainer_key_bytes: Vec<u8> = rlp.val_at(3)?;

        maintainer_raw_key_buf.copy_from_slice(&maintainer_key_bytes);

        let maintainer: VerifyingKey = VerifyingKey::from_bytes(&maintainer_raw_key_buf)
            .map_err(|_| DecoderError::RlpExpectedToBeData)
            .unwrap();

        // Parse archive url
        let raw_archive_url: String = rlp.val_at(4)?;

        let archive_url = Url::parse(raw_archive_url.as_str()).unwrap();

        // Parse integrity struct
        let raw_package_integrity = rlp.list_at(5)?;

        let package_integrity: PackageIntegrity = rlp::decode(&raw_package_integrity)?;

        // Parse signature

        let mut sig_buf: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];

        let sig_bytes: Vec<u8> = rlp.val_at(6)?;

        sig_buf.copy_from_slice(&sig_bytes);

        let sig = Signature::from_bytes(&sig_buf);

        // Build package
        let package = Self {
            name,
            version,
            status,
            maintainer,
            archive_url,
            integrity: package_integrity,
            sig: Some(sig),
        };

        Ok(package)
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} ( Status : {}, Maintainer : {} )",
            self.name,
            self.version,
            self.status,
            hex::encode_upper(self.maintainer)
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ed25519::signature::{rand_core::OsRng, SignerMut};
    use ed25519_dalek::SigningKey;
    use serde_json::json;
    use std::any::{type_name, type_name_of_val};

    use crate::test_utils::package::tests::{create_package_with_sig, create_package_without_sig};

    use super::*;

    /**
     * It should encode and decode package to RLP
     */
    #[test]
    fn test_package_rlp_encode_decode() -> Result<(), Box<dyn std::error::Error>> {
        let package = create_package_with_sig()?;

        let encoded_package = rlp::encode(&package);

        let decoded_package = PackageBuilder::from_rlp(&encoded_package)?.build();

        Ok(())
    }

    /**
     * It should throw error if no signature when encoding to RLP
     */
    #[test]
    #[should_panic(expected = "Signature must be attached to package when encoding it")]
    fn test_package_rlp_sig_err() {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);

        let expected_maintainer = key.verifying_key();

        let package = create_package_without_sig(&expected_maintainer).unwrap();

        rlp::encode(&package);

        ()
    }

    /**
     * It should serialize and deserialize json-encoded package without panic
     */
    #[test]
    fn test_package_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let package = create_package_with_sig()?;

        let json_encoded_package = serde_json::to_string(&package)?;

        let _decoded_package: Package = serde_json::from_str(json_encoded_package.as_str())?;

        Ok(())
    }

    /**
     * It should return error when serializing package with sig missing
     */
    #[test]
    fn test_package_serialization_sig_missing() -> Result<(), Box<dyn std::error::Error>> {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);

        let expected_maintainer = key.verifying_key();

        let package = create_package_without_sig(&expected_maintainer).unwrap();

        let json_encoded_package_result = serde_json::to_string(&package);

        assert_eq!(
            json_encoded_package_result.unwrap_err().to_string(),
            "Signature must be attached to package when serializing it"
        );

        Ok(())
    }

    /**
     * It should display package
     */
    #[test]
    fn test_package_display() -> Result<(), Box<dyn std::error::Error>> {
        let package = create_package_with_sig()?;

        let expected_output = format!(
            "{}:{} ( Status : {}, Maintainer : {} )",
            package.name,
            package.version,
            package.status,
            hex::encode_upper(package.maintainer)
        );

        let package_display = format!("{}", package);

        assert_eq!(package_display, expected_output);
        Ok(())
    }

    /**
     * It should compute rlp data integrity
     */
    #[test]
    fn test_rlp_data_integrity() -> Result<(), Box<dyn std::error::Error>> {
        let package = create_package_with_sig()?;

        let encoded_package_integrity = rlp::encode(&package.integrity);
        let mut stream = rlp::RlpStream::new();

        let encoded_status = package.status.clone() as u8;
        stream
            // Package name
            .append(&package.name)
            // Package version
            .append(&package.version)
            // Package status
            .append(&encoded_status)
            // Package maintainer
            .append(&package.maintainer.to_bytes().as_slice())
            // Package archive urls
            .append(&package.archive_url.as_str())
            // Package integrity
            .append_list(&encoded_package_integrity);

        let mut hasher = Sha256::new();

        hasher.update(stream.as_raw().to_vec());

        let expected_hash = hasher.finalize().to_vec();

        let data_integrity = package.compute_data_integrity();

        assert_eq!(expected_hash, data_integrity);

        Ok(())
    }

    /**
     * It should get builder
     */
    #[test]
    fn test_return_package_builder() {
        let package_builder = Package::builder();

        assert_eq!(
            type_name_of_val(&package_builder),
            type_name::<PackageBuilder>()
        );
    }
}
