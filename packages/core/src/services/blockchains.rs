use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use log::{debug, trace};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};

use crate::{
    blockchains::{
        blockchain::BlockchainClient, errors::blockchain_error::BlockchainError,
        hedera::blockchain_client::HederaBlockchain,
    },
    db::{
        documents::blockchain_document_builder::BlockchainDocumentBuilder,
        traits::repository::Repository,
    },
    packages::package::Package,
};

use super::{db::blockchains_repository::BlockchainsRepository, packages::PackagesService};

pub struct BlockchainsService {
    blockchains_clients: Arc<Mutex<Vec<Arc<Box<dyn BlockchainClient>>>>>,
    selected_client: Arc<Mutex<Option<usize>>>, // TODO : change to ref
    blockchains_repository: Arc<BlockchainsRepository>,
    packages_service: Arc<PackagesService>,
}

impl BlockchainsService {
    /**
     * Create new blockchains service
     */
    pub async fn new(
        blockchains_repository: &Arc<BlockchainsRepository>,
        packages_service: &Arc<PackagesService>,
    ) -> Self {
        let instance = Self {
            blockchains_repository: Arc::clone(&blockchains_repository),
            blockchains_clients: Arc::new(Mutex::new(Vec::new())),
            selected_client: Arc::new(Mutex::new(None)),
            packages_service: Arc::clone(&packages_service),
        };

        instance
    }

    /**
     * Initialize blockchains
     */
    pub async fn init_blockchains(&self) {
        let clients: Vec<Box<dyn BlockchainClient>> =
            vec![Box::new(HederaBlockchain::from("4991716"))];

        for client in clients {
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
                    .set_label(client.get_label())
                    .set_last_synchronization(last_sync.to_string())
                    .build();
                self.blockchains_repository.create(&doc).await;
                debug!("Done registering blockchain !");
            }

            self.blockchains_clients.lock().await.push(Arc::new(client));
        }
    }

    /**
     * Get available clients
     */
    pub fn get_clients(&self) -> Arc<Mutex<Vec<Arc<Box<dyn BlockchainClient>>>>> {
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
            .set_label(client.get_label())
            .set_last_synchronization(client.get_last_sync().await.to_string())
            .build();

        self.blockchains_repository.update(&doc).await;

        debug!("Done updating package manager from blockchain !");

        Ok(())
    }

    /**
     * Find package
     * TODO : move to packages service
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
