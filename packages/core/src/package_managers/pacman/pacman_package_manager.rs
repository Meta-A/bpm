use crate::package_managers::traits::package_manager::PackageManager;
use async_trait::async_trait;
use log::debug;
use std::{
    io::Cursor,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use tempfile::tempdir;

pub struct PacmanPackageManager {}

impl PacmanPackageManager {
    /**
     * Get package url location
     */
    fn get_package_url(&self, package_name: &String) -> Result<String, Box<dyn std::error::Error>> {
        debug!("Fetching package url for {}...", package_name);
        let output = Command::new("pacman")
            .arg("-Spdd")
            .arg(&package_name)
            .stdout(Stdio::piped())
            .output()?;

        let package_url = String::from_utf8(output.stdout)?.trim().to_string();

        debug!("Done fetching package url : {}", package_url);

        Ok(package_url)
    }

    /**
     * Fetch package archive
     */
    async fn fetch_archive(
        &self,
        package_url: &String,
        temp_dir_path: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let package_path = PathBuf::from(package_url);

        let package_path_components = package_path.components();

        let package_filename = package_path_components
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();

        let temp_package_path = temp_dir_path.join(package_filename);

        debug!("Writing package at {}...", temp_package_path.display());

        // Fetch package, save it

        let response = reqwest::get(package_url).await?;

        let mut file = std::fs::File::create(&temp_package_path)?;
        let mut content = Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;

        debug!("Done writing package !");

        Ok(temp_package_path)
    }

    // Uncompresses ZST archive
    //fn uncompress_archive(
    //    &self,
    //    compressed_archive_path: &Path,
    //) -> Result<PathBuf, Box<dyn std::error::Error>> {
    //    debug!("Uncompressiong {}...", compressed_archive_path.display());
    //    let zst_file = File::open(compressed_archive_path)?;
    //    let extension = ".zst";
    //
    //    // Remove last extension
    //    let mut raw_output_path = compressed_archive_path.to_str().unwrap();
    //    raw_output_path = &raw_output_path[0..raw_output_path.len() - extension.len()];
    //
    //    let output_path = PathBuf::from(&raw_output_path);
    //
    //    let output_file = File::create(&output_path)?;
    //
    //    zstd::stream::copy_decode(zst_file, output_file)?;
    //
    //    debug!("Done uncompressiong archive !");
    //
    //    Ok(output_path)
    //}

    // Extract tar package
    //fn extract_archive(&self, archive_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    //    debug!("Extracting archive {}...", archive_path.to_str().unwrap());
    //    let archive_file = File::open(archive_path)?;
    //
    //    let mut archive_path_components = archive_path.components();
    //    archive_path_components.next_back();
    //
    //    let archive_content_dest_path = archive_path_components.as_path();
    //
    //    let mut archive = Archive::new(archive_file);
    //    archive.unpack(archive_content_dest_path)?;
    //
    //    debug!("Done extracting archive !");
    //
    //    Ok(archive_content_dest_path.to_path_buf())
    //}
}

#[async_trait]
impl PackageManager for PacmanPackageManager {
    /**
     * Fetch package content ( binaries, manpages... )
     */
    async fn fetch_package_archive(
        &self,
        package_name: &String,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        debug!("Fetching package binaries...");

        let temp_package_dir = tempdir()?;

        let temp_package_dir_path = temp_package_dir.path();

        let package_url = self.get_package_url(&package_name)?;

        // Download package
        let compressed_archive_path = self
            .fetch_archive(&package_url, temp_package_dir_path)
            .await?;

        debug!("Done fetching package content !");

        Ok(compressed_archive_path)
    }
}

#[cfg(test)]
mod tests {

    use crate::packages::{package::PackageIntegrity, package_builder::PackageBuilder};

    use super::*;

    /**
     * It should get package url using pacman
     */
    #[test]
    fn test_get_package_url() -> Result<(), Box<dyn std::error::Error>> {
        let package_manager = PacmanPackageManager {};

        let package_name_mock = String::from("neofetch");

        let url = package_manager.get_package_url(&package_name_mock)?;

        assert_eq!(url.contains(package_name_mock.as_str()), true);

        Ok(())
    }

    /**
     * It should fetch compressed package archive
     */
    #[tokio::test]
    async fn test_fetch_package_archive() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;

        let package_manager = PacmanPackageManager {};

        let package_name_mock = String::from("neofetch");

        let url = package_manager.get_package_url(&package_name_mock)?;

        let compressed_archive_path = package_manager.fetch_archive(&url, temp.path()).await?;

        assert_eq!(compressed_archive_path.as_path().exists(), true);

        Ok(())
    }

    /**
     * It should fetch package content
     */
    #[tokio::test]
    async fn test_fetch_package_content() {
        let package_manager = PacmanPackageManager {};

        let package_name_mock = String::from("neofetch");

        package_manager
            .fetch_package_archive(&package_name_mock)
            .await
            .unwrap();

        ()
    }
}
