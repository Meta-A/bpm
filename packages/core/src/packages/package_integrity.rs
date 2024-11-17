use polodb_core::bson::{spec::BinarySubtype, Binary, Bson, Document};
use rlp::{Decodable, Encodable};

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

impl Into<Bson> for PackageIntegrity {
    fn into(self) -> Bson {
        let mut doc = Document::new();

        doc.insert("algorithm", &self.algorithm);

        let archive_hash = Binary {
            subtype: BinarySubtype::Generic,
            bytes: self.archive_hash.to_vec(),
        };
        doc.insert("archive_hash", archive_hash);

        Bson::Document(doc)
    }
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