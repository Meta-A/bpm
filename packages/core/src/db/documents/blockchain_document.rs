use super::blockchain_document_builder::BlockchainDocumentBuilder;
use polodb_core::bson::{Bson, Document};

#[derive(serde::Serialize, serde::Deserialize, Debug)]

pub struct BlockchainDocument {
    pub label: String,
    pub last_synchronization: String,
}

impl BlockchainDocument {
    pub fn builder() -> BlockchainDocumentBuilder {
        BlockchainDocumentBuilder::default()
    }
}

impl Into<Bson> for &BlockchainDocument {
    fn into(self) -> Bson {
        let mut doc = Document::new();

        doc.insert("label", &self.label);
        doc.insert("last_synchronization", &self.last_synchronization);

        Bson::Document(doc)
    }
}
