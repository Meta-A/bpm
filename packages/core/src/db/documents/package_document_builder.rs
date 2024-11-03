use crate::packages::package::Package;

use super::package_document::PackageDocument;

pub struct PackageDocumentBuilder {
    package: Option<Package>,
    blockchain_label: Option<String>,
}

impl PackageDocumentBuilder {
    pub fn set_blockchain_label(&mut self, label: String) -> &mut Self {
        self.blockchain_label = Some(label);

        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.blockchain_label = None;

        self
    }

    pub fn from_document(doc: &PackageDocument) -> Self {
        let instance = Self {
            package: None, // TODO : Implement
            blockchain_label: Some(doc.blockchain_label.clone()),
        };

        instance
    }

    //pub fn build(&mut self) -> PackageDocument {
    //    let doc = PackageDocument {
    //        package: None, // TODO: Implement
    //        blockchain_label: self
    //            .blockchain_label
    //            .clone()
    //            .expect("Blockchain label must be set"),
    //    };
    //
    //    self.reset();
    //
    //    doc
    //}
}

impl Default for PackageDocumentBuilder {
    fn default() -> Self {
        let instance = Self {
            package: None,
            blockchain_label: None,
        };

        instance
    }
}
