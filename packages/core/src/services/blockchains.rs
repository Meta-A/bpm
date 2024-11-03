use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use log::debug;
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};

use crate::{
    blockchains::{blockchain::BlockchainClient, hedera::blockchain_client::HederaBlockchain},
    db::{
        documents::blockchain_document_builder::BlockchainDocumentBuilder,
        traits::repository::Repository,
    },
    packages::package::Package,
};

use super::db::blockchains_repository::BlockchainsRepository;

pub struct BlockchainsService {
    blockchains_clients: Arc<Mutex<Vec<Arc<Box<dyn BlockchainClient>>>>>,
    selected_client: Arc<Mutex<Option<usize>>>, // TODO : change to ref
    blockchains_repository: Arc<BlockchainsRepository>,
}

impl BlockchainsService {
    /**
     * Initialize blockchains
     */
    pub async fn init_blockchains(&self) {
        let clients: Vec<Box<dyn BlockchainClient>> =
            vec![Box::new(HederaBlockchain::from("4991716"))];

        for client in clients {
            let exists = self
                .blockchains_repository
                .exists_by_key(&client.get_label())
                .await;

            if exists {
                debug!("Blockchain is already registered");
            } else {
                debug!("Blockchain will be registered...");

                let mut builder = BlockchainDocumentBuilder::default();
                let timestamp = std::time::Instant::now().elapsed().as_millis().to_string();
                let doc = builder
                    .set_label(client.get_label())
                    .set_last_synchronization(timestamp)
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
     * Update package manager from blockchain
     */
    pub async fn update(&self, tx_packages_update: &Sender<Package>) {
        debug!("Updating package manager from blockchain...");
        let (tx_packages, mut rx_packages) = mpsc::channel(1);

        let client = self.get_selected_client().await;
        let task_client = Arc::clone(&client);

        tokio::spawn(async move {
            task_client.read_packages(&tx_packages).await;
        });

        while let Some(package) = rx_packages.recv().await {
            tx_packages_update.send(package).await.unwrap();
        }

        let now = SystemTime::now();
        let epoch_timestamp = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let doc = BlockchainDocumentBuilder::default()
            .set_label(client.get_label())
            .set_last_synchronization(epoch_timestamp.to_string())
            .build();

        self.blockchains_repository.update(&doc).await;

        debug!("Done updating package manager from blockchain !");
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

impl From<&Arc<BlockchainsRepository>> for BlockchainsService {
    fn from(value: &Arc<BlockchainsRepository>) -> Self {
        Self {
            blockchains_repository: Arc::clone(value),
            blockchains_clients: Arc::new(Mutex::new(Vec::new())),
            selected_client: Arc::new(Mutex::new(None)),
        }
    }
}
