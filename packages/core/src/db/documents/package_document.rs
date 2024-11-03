use crate::packages::package::Package;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PackageDocument {
    pub package: Package,
    pub blockchain_label: String,
}
