use log::debug;
use rlp::DecoderError;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{self, Sender};

use super::errors::blockchain_error::BlockchainError;
use crate::packages::{
    package::Package, package_builder::PackageBuilder, utils::signatures::verify_package,
};
use std::fmt::Debug;

#[cfg(test)]
use mockall::automock;

#[async_trait::async_trait]
pub trait BlockchainIO: Send {
    async fn write(&self, data: &[u8]);
    async fn read(&self, tx_data: &Sender<Result<Vec<u8>, BlockchainError>>);
}

#[async_trait::async_trait]
#[cfg_attr(test, automock)]
pub trait BlockchainClient: Sync + Send + Debug {
    /**
     * Write package
     */
    async fn write_package(&self, package: &Package) {
        let io = self.create_io().await;
        debug!("Writing package {} to blockchain...", package.name);

        let encoded_package = rlp::encode(package);
        io.write(&encoded_package).await;

        debug!("Done writing package {} to blockchain !", package.name);
    }

    /**
     * Read packages from blockchain
     */
    async fn read_packages(
        &self,
        tx_packages: &Sender<Result<Package, BlockchainError>>,
    ) -> Result<(), BlockchainError> {
        let io = self.create_io().await;

        let (tx_raw_bytes, mut rx_raw_bytes) = mpsc::channel(1);

        tokio::spawn(async move {
            io.read(&tx_raw_bytes).await;
        });

        while let Some(raw_bytes_res) = rx_raw_bytes.recv().await {
            let raw_bytes = raw_bytes_res?;
            let package_parsing_result: Result<PackageBuilder, DecoderError> =
                PackageBuilder::from_rlp(raw_bytes.as_slice());

            let mut builder = match package_parsing_result {
                Ok(builder) => builder,
                Err(_) => {
                    debug!("Package could not be parsed, skipping",);
                    continue;
                }
            };

            let untrusted_package = builder.build();

            let signature_verification = verify_package(&untrusted_package);

            let trusted_package = match signature_verification {
                Some(trusted_package) => trusted_package,
                None => {
                    debug!("Package signature is wrong, skipping");
                    continue;
                }
            };

            tx_packages.send(Ok(trusted_package.clone())).await.unwrap();
        }

        let current_time = SystemTime::now();
        let epoch_timestamp = current_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        self.set_last_sync(epoch_timestamp).await;

        Ok(())
    }

    /**
     * Get network
     */
    fn get_network(&self) -> String;

    /**
     * Get label
     */
    fn get_label(&self) -> String;

    /**
     * Create blockchain IO
     */
    async fn create_io(&self) -> Box<dyn BlockchainIO>;

    /**
     * Set last sync
     */
    async fn set_last_sync(&self, last_sync: u64);

    /**
     * Get last sync
     */
    async fn get_last_sync(&self) -> u64;
}

impl std::fmt::Display for dyn BlockchainClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_label())?;
        Ok(())
    }
}
