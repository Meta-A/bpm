use super::blockchain_document::BlockchainDocument;

pub struct BlockchainDocumentBuilder {
    label: Option<String>,
    last_synchronization: Option<String>,
}

impl BlockchainDocumentBuilder {
    pub fn set_label(&mut self, label: String) -> &mut Self {
        self.label = Some(label);

        self
    }

    pub fn set_last_synchronization(&mut self, timestamp: String) -> &mut Self {
        self.last_synchronization = Some(timestamp);

        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.label = None;
        self.last_synchronization = None;

        self
    }

    pub fn from_document(doc: &BlockchainDocument) -> Self {
        let instance = Self {
            label: Some(doc.label.clone()),
            last_synchronization: Some(doc.last_synchronization.clone()),
        };

        instance
    }

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
