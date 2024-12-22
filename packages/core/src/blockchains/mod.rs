use std::sync::Arc;

use blockchain::BlockchainClient;
use hedera::blockchain_client::HederaBlockchain;

pub mod blockchain;
pub mod hedera;

pub mod errors;

#[cfg(not(tarpaulin_include))]
pub fn get_available_clients() -> Vec<Arc<Box<dyn BlockchainClient>>> {
    vec![Arc::new(Box::new(HederaBlockchain::from("4991716")))]
}
