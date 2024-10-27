/**
 * Trait used to get information about blockchain
 */
pub trait BlockchainInformation {
    fn get_net(&self) -> String;
}
