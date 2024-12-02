use crate::package_managers::{
    errors::package_manager_error::PackageManagerError, traits::package_manager::PackageManager,
};
use log::debug;
use std::{
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};
use url::Url;

use tempfile::tempdir;

pub struct PacmanPackageManager;

#[cfg(not(tarpaulin_include))] // TODO : Figure out way to test on multiple envs
impl PacmanPackageManager {
    /**
     * Install using local archive
     */
    fn install_archive(&self, archive_path: &PathBuf) -> Result<(), PackageManagerError> {
        debug!(
            "Install archive using pacman ( location : {} )",
            archive_path.display()
        );
        let pacman_process = Command::new("pacman")
            .args(["-U", archive_path.to_str().unwrap(), "--noconfirm"])
            .spawn()
            .map_err(|e| PackageManagerError::InstallationError(e.to_string()))?;

        let output = pacman_process
            .wait_with_output()
            .map_err(|e| PackageManagerError::InstallationError(e.to_string()))?;

        if !output.status.success() {
            let output_str = String::from_utf8(output.stderr).unwrap();
            Err(PackageManagerError::InstallationError(output_str))
        } else {
            debug!(
                "Done installing archive using pacman ( location : {} ) !",
                archive_path.display()
            );

            Ok(())
        }
    }

    /**
     * Fetch package archive
     */
    async fn fetch_archive(
        &self,
        package_url: &Url,
        temp_dir_path: &Path,
    ) -> Result<PathBuf, PackageManagerError> {
        let package_path = PathBuf::from(package_url.as_str());

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

        let response = reqwest::get(package_url.as_str())
            .await
            .map_err(|_| PackageManagerError::DownloadError)?;

        let mut file = std::fs::File::create(&temp_package_path)
            .map_err(|_| PackageManagerError::DownloadError)?;

        let mut content = Cursor::new(
            response
                .bytes()
                .await
                .map_err(|_| PackageManagerError::DownloadError)?,
        );

        std::io::copy(&mut content, &mut file).map_err(|_| PackageManagerError::DownloadError)?;

        debug!("Done writing package !");

        Ok(temp_package_path)
    }
}

#[async_trait::async_trait]
#[cfg(not(tarpaulin_include))] // TODO : Figure out way to test on multiple envs
impl PackageManager for PacmanPackageManager {
    /**
     * Get package manager name
     */
    fn get_name(&self) -> String {
        String::from("pacman")
    }

    /**
     * Fetch package content ( binaries, manpages... )
     */
    async fn install_from_url(&self, package_url: &Url) -> Result<PathBuf, PackageManagerError> {
        debug!(
            "Installing from url (location: {})...",
            package_url.to_string()
        );

        let temp_package_dir =
            tempdir().map_err(|e| PackageManagerError::InstallationError(e.to_string()))?;

        let temp_package_dir_path = temp_package_dir.path();

        // Download package
        let compressed_archive_path = self
            .fetch_archive(package_url, temp_package_dir_path)
            .await?;

        self.install_archive(&compressed_archive_path)?;

        debug!("Done installing package from url !");

        Ok(compressed_archive_path)
    }

    /**
     * Remove package using pacman
     */
    async fn remove(&self, package_name: &String) -> Result<(), PackageManagerError> {
        let pacman_process = Command::new("pacman")
            .args(["-Rsn", package_name.as_str(), "--noconfirm"])
            .spawn()
            .map_err(|e| PackageManagerError::RemovalError(e.to_string()))?;

        let output = pacman_process
            .wait_with_output()
            .map_err(|e| PackageManagerError::RemovalError(e.to_string()))?;

        if !output.status.success() {
            let output_str = String::from_utf8(output.stderr).unwrap();
            Err(PackageManagerError::RemovalError(output_str))
        } else {
            debug!("Done removing package {} using pacman !", package_name);

            Ok(())
        }
    }
}

impl Default for PacmanPackageManager {
    fn default() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {}
