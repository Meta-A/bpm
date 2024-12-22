use super::blockchain_document::BlockchainDocument;

#[derive(Debug)]
pub struct BlockchainDocumentBuilder {
    label: Option<String>,
    last_synchronization: Option<String>,
}

impl BlockchainDocumentBuilder {
    /**
     * Set label
     */
    pub fn set_label(&mut self, label: &String) -> &mut Self {
        self.label = Some(label.clone());

        self
    }

    /**
     * Set blockchain last synchronization
     */
    pub fn set_last_synchronization(&mut self, timestamp: &String) -> &mut Self {
        self.last_synchronization = Some(timestamp.clone());

        self
    }

    /**
     * Reset builder
     */
    pub fn reset(&mut self) -> &mut Self {
        self.label = None;
        self.last_synchronization = None;

        self
    }

    /**
     * Build from document
     */
    pub fn from_document(doc: &BlockchainDocument) -> Self {
        let instance = Self {
            label: Some(doc.label.clone()),
            last_synchronization: Some(doc.last_synchronization.clone()),
        };

        instance
    }

    /**
     * Build document
     */
    pub fn build(&mut self) -> BlockchainDocument {
        let doc = BlockchainDocument {
            label: self.label.clone().expect("Label must be set"),
            last_synchronization: self
                .last_synchronization
                .clone()
                .expect("Last synchronization must be set"),
        };

        self.reset();

        doc
    }
}

impl Default for BlockchainDocumentBuilder {
    fn default() -> Self {
        let instance = Self {
            label: None,
            last_synchronization: None,
        };

        instance
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_blockchain_build() {
        let mut builder = BlockchainDocumentBuilder::default();

        let expected_label = "hedera";
        let expected_last_synchronization = "1704067200";

        let doc = builder
            .set_label(&expected_label.to_string())
            .set_last_synchronization(&expected_last_synchronization.to_string())
            .build();

        assert_eq!(doc.label, expected_label);
        assert_eq!(doc.last_synchronization, expected_last_synchronization);
    }

    #[test]
    fn test_blockchain_reset() {
        let mut builder = BlockchainDocumentBuilder::default();

        let expected_label = "hedera";
        let expected_last_synchronization = "1704067200";

        let doc = builder
            .set_label(&expected_label.to_string())
            .set_last_synchronization(&expected_last_synchronization.to_string())
            .reset();

        assert_eq!(doc.label, None);
        assert_eq!(doc.last_synchronization, None);
    }

    #[test]
    fn test_blockchain_build_from_document() {
        let mut builder = BlockchainDocumentBuilder::default();

        let label_mock = "hedera";
        let last_sync_mock = "1704067200";

        let doc = builder
            .set_label(&label_mock.to_string())
            .set_last_synchronization(&last_sync_mock.to_string())
            .build();

        let new_doc = BlockchainDocumentBuilder::from_document(&doc).build();

        assert_eq!(new_doc.label, doc.label);
        assert_eq!(new_doc.last_synchronization, doc.last_synchronization);
    }
}
