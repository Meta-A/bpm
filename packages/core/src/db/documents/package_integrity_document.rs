use polodb_core::bson::{Bson, Document};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PackageIntegrityDocument {
    pub algorithm: String,
    pub archive_hash: String,
}

impl Into<Bson> for &PackageIntegrityDocument {
    fn into(self) -> Bson {
        let mut doc = Document::new();

        doc.insert("algorithm", &self.algorithm);

        doc.insert("archive_hash", &self.archive_hash);

        Bson::Document(doc)
    }
}
