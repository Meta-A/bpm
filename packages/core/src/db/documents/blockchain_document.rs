use super::blockchain_document_builder::BlockchainDocumentBuilder;
use polodb_core::bson::{Bson, Document};

/**
 * Represents how blockchain is stored in DB
 */

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct BlockchainDocument {
    pub label: String,
    pub last_synchronization: String,
}

impl BlockchainDocument {
    /**
     * Return associated builder
     */
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_return_builder() {
        let builder = BlockchainDocument::builder();

        assert_eq!(
            std::any::type_name::<BlockchainDocumentBuilder>(),
            std::any::type_name_of_val(&builder)
        );
    }

    #[test]
    fn test_should_convert_to_bson() {
        let expected_label = "foo";
        let expected_last_sync = "1704067200";
        let doc = BlockchainDocument {
            label: expected_label.to_string(),
            last_synchronization: expected_last_sync.to_string(),
        };

        let bson_repr: Bson = (&doc).into();
        let bson_doc = bson_repr.as_document().unwrap();

        assert_eq!(doc.label, bson_doc.get_str("label").unwrap());
        assert_eq!(
            doc.last_synchronization,
            bson_doc.get_str("last_synchronization").unwrap()
        );
    }
}
