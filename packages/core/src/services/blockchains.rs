use std::sync::Arc;

use log::{debug, trace};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    blockchains::{blockchain::BlockchainClient, errors::blockchain_error::BlockchainError},
    db::{
        documents::blockchain_document_builder::BlockchainDocumentBuilder,
        traits::repository::Repository,
    },
    packages::package::Package,
    types::asynchronous::AsyncMutex,
};

use super::{db::blockchains_repository::BlockchainsRepository, packages::PackagesService};

#[cfg(test)]
use mockall::automock;

pub struct BlockchainsService {
    blockchains_clients: Arc<AsyncMutex<Vec<Arc<Box<dyn BlockchainClient>>>>>,
    selected_client: Arc<AsyncMutex<Option<usize>>>, // TODO : change to ref
    blockchains_repository: Arc<BlockchainsRepository>,
    packages_service: Arc<PackagesService>,
}

#[cfg_attr(test, automock)]
impl BlockchainsService {
    /**
     * Create new blockchains service
     */
    pub async fn new(
        available_blockchains: &Vec<Arc<Box<dyn BlockchainClient>>>,
        blockchains_repository: &Arc<BlockchainsRepository>,
        packages_service: &Arc<PackagesService>,
    ) -> Self {
        let instance = Self {
            blockchains_repository: Arc::clone(&blockchains_repository),
            blockchains_clients: Arc::new(AsyncMutex::new(available_blockchains.clone())),
            selected_client: Arc::new(AsyncMutex::new(None)),
            packages_service: Arc::clone(&packages_service),
        };

        instance.init_blockchains().await;

        instance
    }

    /**
     * Initialize blockchains
     */
    pub async fn init_blockchains(&self) {
        let clients = self.blockchains_clients.lock().await;

        for client in clients.iter() {
            let blockchain_document_opt = self
                .blockchains_repository
                .read_by_key(&client.get_label())
                .await;

            let exists = blockchain_document_opt.is_some();

            if exists {
                debug!("Blockchain is already registered");
                let blockchain_document =
                    blockchain_document_opt.expect("Blockchain document should have been defined");

                let last_sync: u64 = blockchain_document
                    .last_synchronization
                    .parse()
                    .expect("Could not parse last sync timestamp from blockchain document");

                client.set_last_sync(last_sync).await;
            } else {
                debug!("Blockchain will be registered...");

                let mut builder = BlockchainDocumentBuilder::default();

                let last_sync = 0;

                let doc = builder
                    .set_label(&client.get_label())
                    .set_last_synchronization(&last_sync.to_string())
                    .build();
                self.blockchains_repository.create(&doc).await;
                debug!("Done registering blockchain !");
            }
        }
    }

    /**
     * Get available clients
     */
    pub fn get_clients(&self) -> Arc<AsyncMutex<Vec<Arc<Box<dyn BlockchainClient>>>>> {
        Arc::clone(&self.blockchains_clients)
    }

    /**
     * Set current client
     */
    pub async fn set_client(&self, client_idx: usize) {
        let mut selected_client_lock = self.selected_client.lock().await;

        *selected_client_lock = Some(client_idx);
    }

    /**
     * Get current client
     */
    pub async fn get_selected_client(&self) -> Arc<Box<dyn BlockchainClient>> {
        let clients = self.blockchains_clients.lock().await;

        let selected_id = self
            .selected_client
            .lock()
            .await
            .expect("Blockchain id must be set in order to get current client");

        let client = clients
            .get(selected_id)
            .expect("Could not find blockchain client");

        Arc::clone(client)
    }

    /**
     * This method is used to process package when updating from blockchain
     */
    async fn process_package_update(
        &self,
        package: &Package,
        selected_client: &Box<dyn BlockchainClient>,
    ) {
        let package_exists = self
            .packages_service
            .exists(&package, selected_client)
            .await;

        if package_exists {
            trace!("Package already exists, updating it...");

            self.packages_service
                .update_package(&package, selected_client)
                .await;

            trace!("Done updating already existing package !");
        } else {
            trace!("Package doesn't exist, adding it...");

            self.packages_service.add(&package, selected_client).await;

            trace!("Done adding new package !");
        }
    }

    /**
     * Update package manager from blockchain
     */
    pub async fn update(
        &self,
        tx_packages_update: &Sender<Package>,
    ) -> Result<(), BlockchainError> {
        debug!("Updating package manager from blockchain...");
        let (tx_packages, mut rx_packages): (
            Sender<Result<Package, BlockchainError>>,
            Receiver<Result<Package, BlockchainError>>,
        ) = mpsc::channel(1);

        let client = self.get_selected_client().await;
        let task_client = Arc::clone(&client);

        // Start to read packages from blockchain
        tokio::spawn(async move {
            let task_res = task_client.read_packages(&tx_packages).await;

            match task_res {
                Ok(_) => (),
                Err(e) => {
                    tx_packages.send(Err(e)).await.unwrap();
                    return;
                }
            }
        });

        let selected_client = self.get_selected_client().await;

        // Send notifications to upper scopes
        while let Some(package_res) = rx_packages.recv().await {
            let package = match package_res {
                Ok(package) => package,
                Err(e) => {
                    return Err(e);
                }
            };
            self.process_package_update(&package, &selected_client)
                .await;

            tx_packages_update.send(package).await.unwrap();
        }

        // Update current blockchain's doc to set last sync time to now
        let doc = BlockchainDocumentBuilder::default()
            .set_label(&client.get_label())
            .set_last_synchronization(&client.get_last_sync().await.to_string())
            .build();

        self.blockchains_repository.update(&doc.label, &doc).await;

        debug!("Done updating package manager from blockchain !");

        Ok(())
    }

    /**
     * Find package
     */
    pub async fn find_package(
        &self,
        package_name: &String,
        package_version: &String,
    ) -> Vec<Package> {
        let selected_client = self.get_selected_client().await;
        let matching_packages = self
            .packages_service
            .get_by_release(&package_name, &package_version, &selected_client)
            .await;

        matching_packages
    }

    /**
     * Submit package to blockchain
     */
    pub async fn submit_package(&self, package: &Package) {
        debug!("Submitting package to blockchain IO...");

        let client = self.get_selected_client().await;
        client.write_package(package).await;

        debug!("Done submitting package to blockchain IO !");
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        blockchains::blockchain::MockBlockchainClient,
        services::db::packages_repository::PackagesRepository,
        test_utils::{db::tests::create_test_db, package::tests::create_package_with_sig},
    };
    use mockall::{mock, predicate::*};

    use super::*;

    /**
     * It should get client
     */
    #[tokio::test]
    async fn test_get_clients() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        let expected_label = blockchain_mock.get_label().clone();

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);

        let blockchains_clients_mock = vec![Arc::new(blockchain_client)];

        let blockchains_service = BlockchainsService::new(
            &blockchains_clients_mock,
            &blockchains_repository,
            &packages_service,
        )
        .await;

        let clients = blockchains_service.get_clients();

        let label = clients.lock().await[0].get_label();
        assert_eq!(label, expected_label);

        Ok(())
    }

    /**
     * It should initialize blockchains
     */
    #[tokio::test]
    async fn test_init_blockchains() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        blockchain_mock
            .expect_get_last_sync()
            .returning(|| Box::pin(async { 0 }));

        blockchain_mock
            .expect_set_last_sync()
            .returning(|_| Box::pin(async { println!("set_last_sync mock executed") }));

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);

        let blockchains_clients_mock = vec![Arc::new(blockchain_client)];

        let blockchains_service = BlockchainsService::new(
            &blockchains_clients_mock,
            &blockchains_repository,
            &packages_service,
        )
        .await;

        let mut blockhains_docs_count = blockchains_repository.read_all().await.len();

        assert_eq!(blockhains_docs_count, 1);

        // If blockchain doc already exists it should not add it twice
        blockchains_service.init_blockchains().await;
        blockhains_docs_count = blockchains_repository.read_all().await.len();

        assert_eq!(blockhains_docs_count, 1);

        Ok(())
    }

    /**
     * It should update package
     */
    #[tokio::test]
    async fn test_update_blockchain() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        blockchain_mock
            .expect_get_last_sync()
            .returning(|| Box::pin(async { 0 }));

        let expected_package = create_package_with_sig().unwrap();
        // Return one package mutation
        blockchain_mock
            .expect_read_packages()
            .returning(move |tx_packages| {
                let tx_packages = tx_packages.clone();

                let package = expected_package.clone();

                Box::pin(async move {
                    tx_packages.send(Ok(package.clone())).await.unwrap();
                    Ok(())
                })
            });

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);

        let blockchains_clients_mock = vec![Arc::new(blockchain_client)];

        let blockchains_service = BlockchainsService::new(
            &blockchains_clients_mock,
            &blockchains_repository,
            &packages_service,
        )
        .await;

        blockchains_service.set_client(0).await;

        // Get packages mutations
        let (tx_packages, mut _rx_packages): (Sender<Package>, Receiver<Package>) =
            mpsc::channel(1);
        blockchains_service.update(&tx_packages).await.unwrap();

        _rx_packages.recv().await;

        // Only one mutation should be found now

        let mut packages_docs_count = packages_service.get_all().await.len();

        let expected_packages_count = 1;

        assert_eq!(packages_docs_count, expected_packages_count);

        blockchains_service.update(&tx_packages).await.unwrap();

        _rx_packages.recv().await;

        packages_docs_count = packages_service.get_all().await.len();

        assert_eq!(packages_docs_count, expected_packages_count);

        Ok(())
    }

    /**
     * It should raise BlockchainError
     */
    #[tokio::test]
    async fn test_update_blockchain_error() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        blockchain_mock
            .expect_get_last_sync()
            .returning(|| Box::pin(async { 0 }));

        let expected_error = BlockchainError::NoPackagesData;

        blockchain_mock
            .expect_read_packages()
            .returning(|tx_packages| {
                Box::pin(async move {
                    return Err(BlockchainError::NoPackagesData);
                })
            });

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);

        let blockchains_clients_mock = vec![Arc::new(blockchain_client)];

        let blockchains_service = BlockchainsService::new(
            &blockchains_clients_mock,
            &blockchains_repository,
            &packages_service,
        )
        .await;

        blockchains_service.set_client(0).await;

        // Get packages mutations
        let (tx_packages, mut _rx_packages): (Sender<Package>, Receiver<Package>) =
            mpsc::channel(1);
        let res = blockchains_service.update(&tx_packages).await;

        assert_eq!(res.unwrap_err(), expected_error);

        Ok(())
    }

    /**
     * It should find package by release
     */
    #[tokio::test]
    async fn test_should_find_package_by_release() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        blockchain_mock
            .expect_get_last_sync()
            .returning(|| Box::pin(async { 0 }));

        let package = create_package_with_sig().unwrap();
        let shared_package = package.clone();

        // Return one package mutation
        blockchain_mock
            .expect_read_packages()
            .returning(move |tx_packages| {
                let tx_packages = tx_packages.clone();

                let package = shared_package.clone();

                Box::pin(async move {
                    tx_packages.send(Ok(package.clone())).await.unwrap();
                    Ok(())
                })
            });

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);

        let blockchains_clients_mock = vec![Arc::new(blockchain_client)];

        let blockchains_service = BlockchainsService::new(
            &blockchains_clients_mock,
            &blockchains_repository,
            &packages_service,
        )
        .await;

        blockchains_service.set_client(0).await;

        // Get packages mutations
        let (tx_packages, mut _rx_packages): (Sender<Package>, Receiver<Package>) =
            mpsc::channel(1);

        blockchains_service.update(&tx_packages).await.unwrap();

        let found_packages = blockchains_service
            .find_package(&package.name, &package.version)
            .await;

        assert_eq!(package, found_packages[0]);

        Ok(())
    }

    /**
     * It should submit package
     */
    #[tokio::test]
    async fn test_submit_package() -> Result<(), Box<dyn std::error::Error>> {
        let db_client = create_test_db();

        // Instantiate required resources

        let blockchains_repository = Arc::new(BlockchainsRepository::from(&db_client));
        let packages_repository = Arc::new(PackagesRepository::from(&db_client));

        let packages_service = Arc::new(PackagesService::from(&packages_repository));

        let mut blockchain_mock = MockBlockchainClient::default();

        let expected_submission_calls_count = 1;
        blockchain_mock
            .expect_write_package()
            .times(expected_submission_calls_count)
            .returning(|package| {
                Box::pin(async {
                    println!("Mocked package write...");
                })
            });

        blockchain_mock
            .expect_get_label()
            .returning(|| "MockBlockchain".to_string());

        let blockchain_client: Box<dyn BlockchainClient> = Box::new(blockchain_mock);

        let blockchains_clients_mock = vec![Arc::new(blockchain_client)];

        let blockchains_service = BlockchainsService::new(
            &blockchains_clients_mock,
            &blockchains_repository,
            &packages_service,
        )
        .await;

        blockchains_service.set_client(0).await;

        let package = create_package_with_sig()?;
        blockchains_service.submit_package(&package).await;

        Ok(())
    }
}
