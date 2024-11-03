use async_trait::async_trait;
use log::debug;
use rlp::DecoderError;
use tokio::sync::mpsc::{self, Sender};

use crate::packages::{
    package::Package, package_builder::PackageBuilder, utils::signatures::verify_package,
};

#[async_trait::async_trait]
pub trait BlockchainIO: Send {
    async fn write(&self, data: &[u8]);
    async fn read(&self, tx_data: &Sender<Vec<u8>>);
}

#[async_trait::async_trait]
pub trait BlockchainClient: Sync + Send {
    /**
     * Write package
     */
    async fn write_package(&self, package: &Package) {
        let io = self.create_io();
        debug!("Writing package {} to blockchain...", package.name);

        let encoded_package = rlp::encode(package);
        io.write(&encoded_package).await;

        debug!("Done writing package {} to blockchain !", package.name);
    }

    /**
     * Read packages from blockchain
     */
    async fn read_packages(&self, tx_packages: &Sender<Package>) {
        let io = self.create_io();

        let (tx_raw_bytes, mut rx_raw_bytes) = mpsc::channel(1);

        tokio::spawn(async move {
            io.read(&tx_raw_bytes).await;
        });

        while let Some(raw_bytes) = rx_raw_bytes.recv().await {
            let package_parsing_result: Result<PackageBuilder, DecoderError> =
                PackageBuilder::from_rpl(raw_bytes.as_slice());

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

            tx_packages.send(trusted_package.clone()).await.unwrap();
        }
    }

    /**
     * Get last sync
     */
    fn get_last_sync(&self) -> u64;

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
    fn create_io(&self) -> Box<dyn BlockchainIO>;
}

impl std::fmt::Display for dyn BlockchainClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_label())?;
        Ok(())
    }
}
