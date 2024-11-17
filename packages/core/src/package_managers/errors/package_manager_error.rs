use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackageManagerError {
    #[error("Package manager could not download package")]
    DownloadError,
    #[error("Package manager could not install package: {0}")]
    InstallationError(String),

    #[error("Package manager could not remove package: {0}")]
    RemovalError(String),
}
