use core::fmt;
use std::{collections::HashMap, path::PathBuf};

pub type PackageBinariesIntegrityMap = HashMap<PathBuf, String>;
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PackageIntegrity {
    pub algorithm: String,
    // Too expensive on blockchain :(
    //pub binaries: PackageBinariesIntegrityMap,
    pub content_hash: String,
    //pub source_code_hash: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub integrity: PackageIntegrity,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Package information : \n")?;
        write!(f, "\tName : {} \n", self.name)?;
        write!(f, "\tVersion : {} \n", self.version)?;

        write!(f, "\tPackage integrity :\n")?;
        write!(f, "\t\tContent hash : {} \n", self.integrity.content_hash)?;
        write!(f, "\t\tSource code hash :\n")?;

        Ok(())
    }
}
