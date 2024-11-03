use super::package_builder::PackageBuilder;
use base64::prelude::*;
use core::fmt;
use ed25519::pkcs8::spki::der::pem::Base64Encoder;
use ed25519::Signature;
use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use polodb_core::bson::{Binary, Bson, Document};
use rlp::{Decodable, DecoderError, Encodable, RlpStream};
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DEFAULT_PACKAGE_STATUS: PackageStatus = PackageStatus::Fine;

/**
 * Package status
 */

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum PackageStatus {
    NA = 0x00,
    Prohibited = 0x01,
    Outdated = 0x02,
    Fine = 0x03,
    Recommended = 0x04,
    HighlyRecommended = 0x05,
}

impl fmt::Display for PackageStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PackageStatus::NA => write!(f, "NA"),
            PackageStatus::Prohibited => write!(f, "Prohibited"),
            PackageStatus::Outdated => write!(f, "Outdated"),
            PackageStatus::Fine => write!(f, "Fine"),
            PackageStatus::Recommended => write!(f, "Recommended"),
            PackageStatus::HighlyRecommended => write!(f, "Highly recommended"),
        }
    }
}

impl TryFrom<u8> for PackageStatus {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PackageStatus::NA),
            1 => Ok(PackageStatus::Prohibited),
            2 => Ok(PackageStatus::Outdated),
            3 => Ok(PackageStatus::Fine),
            4 => Ok(PackageStatus::Recommended),
            5 => Ok(PackageStatus::HighlyRecommended),
            _ => Err("Invalid value for PackageStatus"),
        }
    }
}

/**
 * Package integrity fields
 */
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PackageIntegrity {
    pub algorithm: String,
    pub archive_hash: Vec<u8>,
    //pub source_code_hash: String,
}

impl Encodable for PackageIntegrity {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_unbounded_list()
            // Algorithm
            .append(&self.algorithm)
            // Archive hash
            .append(&self.archive_hash)
            .finalize_unbounded_list();
    }
}

impl Decodable for PackageIntegrity {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let algorithm: String = rlp.val_at(0)?;
        let archive_hash: Vec<u8> = rlp.val_at(1)?;

        let package_integrity = Self {
            algorithm,
            archive_hash,
        };

        Ok(package_integrity)
    }
}

/**
 * Package
 */
#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub status: PackageStatus,
    pub maintainer: VerifyingKey, // Maintainer is identified by its public key
    pub integrity: PackageIntegrity,
    pub sig: Option<Signature>,
}

// DB encoding
impl Into<Bson> for &Package {
    fn into(self) -> Bson {
        let mut doc = Document::new();

        doc.insert("name", &self.name);

        doc.insert("version", &self.version);

        let encoded_maintainer = BASE64_STANDARD.encode(self.maintainer.to_bytes());
        let maintainer_key = Binary::from_base64(encoded_maintainer, None).unwrap();
        doc.insert("maintainer", maintainer_key);

        let encoded_sig = BASE64_STANDARD.encode(self.sig.unwrap().to_bytes());
        let sig = Binary::from_base64(encoded_sig, None).unwrap();
        doc.insert("sig", sig);

        Bson::Document(doc)
    }
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
            // Package integrity
            .append_list(&encoded_package_integrity);

        stream
    }

    pub fn builder() -> PackageBuilder {
        PackageBuilder::new()
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

        state.serialize_field("integrity", &self.integrity)?;

        let sig = match self.sig {
            Some(v) => v,
            None => panic!("Signature must be attached to package when serializing it"),
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
            Integrity,
            Sig,
        }
        struct PackageVisitor;

        impl<'de> Visitor<'de> for PackageVisitor {
            type Value = Package;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Package")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Package, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let name = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let version = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                // Parse status

                let raw_status: u8 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let status =
                    PackageStatus::try_from(raw_status).map_err(|e| de::Error::custom(e))?;

                // Parse maintainer verifying key
                let mut maintainer_raw_key_buf: [u8; PUBLIC_KEY_LENGTH] = [0; PUBLIC_KEY_LENGTH];

                let maintainer_key_bytes: &[u8] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                maintainer_raw_key_buf.copy_from_slice(&maintainer_key_bytes);

                let maintainer: VerifyingKey = VerifyingKey::from_bytes(&maintainer_raw_key_buf)
                    .map_err(|_| DecoderError::RlpExpectedToBeData)
                    .unwrap();

                // Parse integrity

                let integrity: PackageIntegrity = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;

                // Parse signature

                let mut sig_buf: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];

                let sig_bytes: Vec<u8> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                sig_buf.copy_from_slice(&sig_bytes);

                let sig = Signature::from_bytes(&sig_buf);

                let package = Package {
                    name,
                    version,
                    status,
                    maintainer,
                    integrity,
                    sig: Some(sig),
                };
                Ok(package)
            }

            fn visit_map<V>(self, mut map: V) -> Result<Package, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut version = None;
                let mut status = None;
                let mut maintainer = None;
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

                let integrity = integrity.ok_or_else(|| de::Error::missing_field("integrity"))?;
                let sig = sig.ok_or_else(|| de::Error::missing_field("sig"))?;

                let package = Package {
                    name,
                    version,
                    status,
                    maintainer,
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

        // Parse integrity struct
        let raw_package_integrity = rlp.list_at(4)?;

        let package_integrity: PackageIntegrity = rlp::decode(&raw_package_integrity)?;

        // Parse signature

        let mut sig_buf: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];

        let sig_bytes: Vec<u8> = rlp.val_at(5)?;

        sig_buf.copy_from_slice(&sig_bytes);

        let sig = Signature::from_bytes(&sig_buf);

        // Build package
        let package = Self {
            name,
            version,
            status,
            maintainer,
            integrity: package_integrity,
            sig: Some(sig),
        };

        Ok(package)
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sig = match self.sig {
            Some(v) => hex::encode(v.to_bytes()),
            None => String::from("<No signature attached>"),
        };

        write!(f, "--- Package information ---\n\n")?;
        write!(f, "Name => {} \n", self.name)?;
        write!(f, "Version => {} \n", self.version)?;
        write!(f, "Status => {} \n", self.status)?;

        write!(
            f,
            "Maintainer => {}\n",
            hex::encode_upper(self.maintainer.to_bytes())
        )?;

        write!(f, "Package integrity :\n")?;
        write!(f, "\tAlgorithm => {} \n", self.integrity.algorithm)?;
        write!(
            f,
            "\tArchive hash => {} \n",
            hex::encode(&self.integrity.archive_hash)
        )?;
        write!(f, "\tSource code hash =>\n")?;

        write!(f, "Signature => {}\n", sig)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //It should display package
    //#[test]
    //fn test_display_package() {
    //    let package_integrity_mock = PackageIntegrity {
    //        algorithm: "SHA256".to_string(),
    //        archive_hash: hex::encode(),
    //    };
    //
    //    let package = Package {
    //        name: "foo".to_string(),
    //        version: "1.2.3".to_string(),
    //        integrity: package_integrity_mock,
    //    };
    //
    //    let package_buf = String::from(format!("{package}"));
    //
    //    assert_eq!(package_buf.contains(&package.name), true);
    //    assert_eq!(package_buf.contains(&package.version), true);
    //}
}
