use std::{error::Error, future::Future, pin::Pin};

use crate::packages::package::Package;

/**
 * Represents a blockchain client
 */
#[trait_variant::make(BlockchainClient: Send)]
pub trait LocalBlockchainClient {
    fn get_net(&self) -> &String;

    async fn get_packages(&self) -> Result<(), Box<dyn std::error::Error>>;
    //async fn get_packages(&self) -> Vec<Package>;
}

//pub fn init_blockchains() -> impl FnMut(Box<dyn Blockchain>) {
//    let mut vec: Vec<Box<dyn Blockchain>> = Vec::new();
//
//    move |blockchain: Box<dyn Blockchain>| {
//        vec.push(blockchain);
//    }
//}
