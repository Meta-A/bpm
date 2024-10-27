use crate::{
    package_managers::traits::package_manager::PackageManager, packages::package::Package,
};
use async_trait::async_trait;
use log::{debug, info};
use std::{
    fs::File,
    io::Cursor,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::utils::fs::unix::executables::find_executables;

use tar::Archive;
use tempfile::tempdir;

pub struct PacmanPackageManager {}

impl PacmanPackageManager {
    /**
     * Get package url location
     */
    fn get_package_url(&self, package: &Package) -> Result<String, Box<dyn std::error::Error>> {
        debug!("Fetching package url for {}...", package.name);
        let output = Command::new("pacman")
            .arg("-Spdd")
            .arg(package.name.clone())
            .stdout(Stdio::piped())
            .output()?;

        let package_url = String::from_utf8(output.stdout)?.trim().to_string();

        debug!("Done fetching package url : {}", package_url);

        Ok(package_url)
    }

    /**
     * Fetch package archive
     */
    async fn fetch_package_archive(
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

    /**
     * Uncompresses ZST archive
     */
    fn uncompress_archive(
        &self,
        compressed_archive_path: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        debug!("Uncompressiong {}...", compressed_archive_path.display());
        let zst_file = File::open(compressed_archive_path)?;
        let extension = ".zst";

        // Remove last extension
        let mut raw_output_path = compressed_archive_path.to_str().unwrap();
        raw_output_path = &raw_output_path[0..raw_output_path.len() - extension.len()];

        let output_path = PathBuf::from(&raw_output_path);

        let output_file = File::create(&output_path)?;

        zstd::stream::copy_decode(zst_file, output_file)?;

        debug!("Done uncompressiong archive !");

        Ok(output_path)
    }

    /**
     * Extract tar package
     */
    fn extract_archive(&self, archive_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        debug!("Extracting archive {}...", archive_path.to_str().unwrap());
        let archive_file = File::open(archive_path)?;

        let mut archive_path_components = archive_path.components();
        archive_path_components.next_back();

        let archive_content_dest_path = archive_path_components.as_path();

        let mut archive = Archive::new(archive_file);
        archive.unpack(archive_content_dest_path)?;

        debug!("Done extracting archive !");

        Ok(archive_content_dest_path.to_path_buf())
    }
}

#[async_trait]
impl PackageManager for PacmanPackageManager {
    /**
     * Fetch package content ( binaries, manpages... )
     */
    async fn fetch_package_content(
        &self,
        package: &Package,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Fetching package binaries...");

        let temp_package_dir = tempdir()?;

        let temp_package_dir_path = temp_package_dir.path();

        let package_url = self.get_package_url(&package)?;

        // Download package
        let compressed_archive_path = self
            .fetch_package_archive(&package_url, temp_package_dir_path)
            .await?;

        // Uncompress ZST
        let uncompressed_archive_path = self.uncompress_archive(&compressed_archive_path)?;

        // Uncompress archive

        self.extract_archive(&uncompressed_archive_path)?;

        find_executables(temp_package_dir_path)?;
        info!("Done fetching package content !");

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::packages::package_builder::PackageBuilder;

    use super::*;

    /**
     * It should get package url using pacman
     */
    #[test]
    fn test_get_package_url() -> Result<(), Box<dyn std::error::Error>> {
        let package_manager = PacmanPackageManager {};

        let expected_package = PackageBuilder::new()
            .set_package_name("neofetch".to_string())
            .set_package_version("7.1.0-2".to_string())
            .build();

        let url = package_manager.get_package_url(&expected_package)?;
        assert_eq!(url.contains(expected_package.name.as_str()), true);

        Ok(())
    }

    /**
     * It should fetch compressed package archive
     */
    #[tokio::test]
    async fn test_fetch_package_archive() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let package_manager = PacmanPackageManager {};

        let expected_package = PackageBuilder::new()
            .set_package_name("neofetch".to_string())
            .set_package_version("7.1.0-2".to_string())
            .build();

        let url = package_manager.get_package_url(&expected_package)?;

        let compressed_archive_path = package_manager
            .fetch_package_archive(&url, temp.path())
            .await?;

        assert_eq!(compressed_archive_path.as_path().exists(), true);

        Ok(())
    }

    /**
     * It should uncompress package archive
     */
    #[tokio::test]
    async fn test_uncompress_package_archive() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let package_manager = PacmanPackageManager {};

        let expected_package = PackageBuilder::new()
            .set_package_name("neofetch".to_string())
            .set_package_version("7.1.0-2".to_string())
            .build();

        let url = package_manager.get_package_url(&expected_package)?;

        let compressed_archive_path = package_manager
            .fetch_package_archive(&url, temp.path())
            .await?;

        let uncompressed_archive_path =
            package_manager.uncompress_archive(&compressed_archive_path)?;

        assert_eq!(uncompressed_archive_path.as_path().exists(), true);

        Ok(())
    }

    /**
     * It should extract package archive
     */
    #[tokio::test]
    async fn test_extract_package_archive() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let package_manager = PacmanPackageManager {};

        let temp_dir_path = temp.path();

        let expected_package = PackageBuilder::new()
            .set_package_name("neofetch".to_string())
            .set_package_version("7.1.0-2".to_string())
            .build();

        let exepcted_extracted_dir = "usr";

        let url = package_manager.get_package_url(&expected_package)?;

        let compressed_archive_path = package_manager
            .fetch_package_archive(&url, temp_dir_path)
            .await?;

        let uncompressed_archive_path =
            package_manager.uncompress_archive(&compressed_archive_path)?;

        // Neofetch creates /usr dir once extracted we can test that
        package_manager.extract_archive(&uncompressed_archive_path)?;

        assert_eq!(temp_dir_path.join(exepcted_extracted_dir).exists(), true);

        Ok(())
    }
}
