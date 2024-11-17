use polodb_core::bson::{Bson, Document};

use super::package_integrity_document::PackageIntegrityDocument;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PackageDocument {
    pub name: String,
    pub version: String,
    pub status: i32,
    pub maintainer: String,
    pub archive_url: String,
    pub integrity: PackageIntegrityDocument,
    pub sig: String,
    pub blockchain_label: String,
}

impl Into<Bson> for &PackageDocument {
    fn into(self) -> Bson {
        let mut doc = Document::new();

        doc.insert("name", &self.name);

        doc.insert("version", &self.version);

        doc.insert("status", &self.status);

        doc.insert("maintainer", &self.maintainer);

        doc.insert("archive_url", &self.archive_url);

        let integrity: Bson = (&self.integrity).into();
        doc.insert("integrity", integrity);

        doc.insert("sig", &self.sig);

        doc.insert("blockchain_label", &self.blockchain_label);

        Bson::Document(doc)
    }
}
