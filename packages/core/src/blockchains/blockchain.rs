use log::debug;
use rlp::DecoderError;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc::{self, Sender};

use super::errors::blockchain_error::BlockchainError;
use crate::packages::{
    package::Package, package_builder::PackageBuilder, utils::signatures::verify_package,
};
use std::fmt::Debug;

#[cfg(test)]
use mockall::automock;

#[async_trait::async_trait]
#[cfg_attr(test, automock)]
pub trait BlockchainIO: Sync + Send + Debug {
    async fn write(&self, data: &[u8]);
    async fn read(&self, tx_data: &Sender<Result<Vec<u8>, BlockchainError>>, last_sync: &u64);
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

        let last_sync = self.get_last_sync().await;
        tokio::spawn(async move {
            io.read(&tx_raw_bytes, &last_sync).await;
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
     * Get label
     */
    fn get_label(&self) -> String;

    /**
     * Create blockchain IO
     */
    async fn create_io(&self) -> Arc<Box<dyn BlockchainIO>>;

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

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use tokio::sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    };

    use crate::{
        blockchains::{
            blockchain::{BlockchainClient, BlockchainIO, MockBlockchainIO},
            errors::blockchain_error::BlockchainError,
            hedera::blockchain_client::HederaBlockchain,
        },
        packages::{package::Package, package_builder::PackageBuilder},
        test_utils::package::tests::create_package_with_sig,
    };

    /**
     * It should display blockchain name
     */
    #[tokio::test]
    async fn test_display_blockchain_name() {
        let mut hedera_io_mock = MockBlockchainIO::default();

        hedera_io_mock
            .expect_read()
            .returning(move |_, _| Box::pin(async move {}));

        let hedera_io: Box<dyn BlockchainIO> = Box::new(hedera_io_mock);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::new(hedera_io));

        assert_eq!(
            format!("{}", blockchain_client),
            blockchain_client.get_label()
        );
    }

    /**
     * It should get packages
     */
    #[tokio::test]
    async fn test_should_get_packages() {
        let expected_package = create_package_with_sig().unwrap();

        let mut hedera_io_mock = MockBlockchainIO::default();

        let shared_pkg = expected_package.clone();

        hedera_io_mock
            .expect_read()
            .returning(move |tx_packages, _| {
                let pkg = shared_pkg.clone();
                let tx = tx_packages.clone();
                Box::pin(async move {
                    let encoded_pkg = rlp::encode(&pkg).to_vec();

                    tx.send(Ok(encoded_pkg)).await.unwrap();
                })
            });

        let hedera_io: Box<dyn BlockchainIO> = Box::new(hedera_io_mock);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::new(hedera_io));

        let (tx_packages, mut rx_packages): (
            Sender<Result<Package, BlockchainError>>,
            Receiver<Result<Package, BlockchainError>>,
        ) = tokio::sync::mpsc::channel(1);

        blockchain_client.read_packages(&tx_packages).await.unwrap();

        let package = rx_packages.recv().await.unwrap().unwrap();

        assert_eq!(package, expected_package);
    }

    /**
     * It should skip not parseable packages
     */
    #[tokio::test]
    async fn test_should_skip_not_parseable_packages() {
        let expected_package = create_package_with_sig().unwrap();

        let mut hedera_io_mock = MockBlockchainIO::default();

        let shared_pkg = expected_package.clone();

        hedera_io_mock
            .expect_read()
            .returning(move |tx_packages, _| {
                let pkg = shared_pkg.clone();
                let tx = tx_packages.clone();

                Box::pin(async move {
                    let encoded_pkg = rlp::encode(&pkg).to_vec();

                    tx.send(Ok(Vec::from("foobar"))).await.unwrap();
                    tx.send(Ok(encoded_pkg)).await.unwrap();
                })
            });

        let hedera_io: Box<dyn BlockchainIO> = Box::new(hedera_io_mock);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::new(hedera_io));

        let (tx_packages, mut rx_packages): (
            Sender<Result<Package, BlockchainError>>,
            Receiver<Result<Package, BlockchainError>>,
        ) = tokio::sync::mpsc::channel(1);

        blockchain_client.read_packages(&tx_packages).await.unwrap();

        let package = rx_packages.recv().await.unwrap().unwrap();

        assert_eq!(package, expected_package);
    }

    /**
     * It should skip package with wrong signature
     */
    #[tokio::test]
    async fn test_should_skip_forged_packages() {
        let mut forged_package = create_package_with_sig().unwrap();
        forged_package = PackageBuilder::from_package(&forged_package)
            .set_name(&String::from("baz"))
            .build();

        let expected_package = create_package_with_sig().unwrap();

        let mut hedera_io_mock = MockBlockchainIO::default();

        let shared_pkg = expected_package.clone();

        hedera_io_mock
            .expect_read()
            .returning(move |tx_packages, _| {
                let pkg = shared_pkg.clone();
                let forged_pkg = forged_package.clone();
                let tx = tx_packages.clone();

                Box::pin(async move {
                    let encoded_forged_pkg = rlp::encode(&forged_pkg).to_vec();
                    let encoded_pkg = rlp::encode(&pkg).to_vec();

                    tx.send(Ok(encoded_forged_pkg)).await.unwrap();
                    tx.send(Ok(encoded_pkg)).await.unwrap();
                })
            });

        let hedera_io: Box<dyn BlockchainIO> = Box::new(hedera_io_mock);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::new(hedera_io));

        let (tx_packages, mut rx_packages): (
            Sender<Result<Package, BlockchainError>>,
            Receiver<Result<Package, BlockchainError>>,
        ) = tokio::sync::mpsc::channel(1);

        blockchain_client.read_packages(&tx_packages).await.unwrap();

        let package = rx_packages.recv().await.unwrap().unwrap();

        assert_eq!(package, expected_package);
    }

    /**
     * It should write package
     */
    #[tokio::test]
    async fn test_should_write_package() {
        let expected_package = create_package_with_sig().unwrap();

        let mut hedera_io_mock = MockBlockchainIO::default();

        let actual_written_package = Arc::new(Mutex::new(None));
        let shared_package: Arc<Mutex<Option<Package>>> = Arc::clone(&actual_written_package);

        hedera_io_mock
            .expect_write()
            .times(1)
            .returning(move |written_bytes| {
                let bytes = Vec::from(written_bytes);
                let pkg_clone = Arc::clone(&shared_package);
                Box::pin(async move {
                    let mut pkg = pkg_clone.lock().await;
                    *pkg = Some(PackageBuilder::from_rlp(&bytes.as_slice()).unwrap().build());
                })
            });

        let hedera_io: Box<dyn BlockchainIO> = Box::new(hedera_io_mock);

        let blockchain_client: Box<dyn BlockchainClient> =
            Box::new(HederaBlockchain::new(hedera_io));

        blockchain_client.write_package(&expected_package).await;

        let actual_written_package = actual_written_package
            .lock()
            .await
            .as_ref()
            .unwrap()
            .clone();

        assert_eq!(expected_package, actual_written_package);
    }
}
