use core::error;

use super::block::Block;

/**
 * Represents a blockchain
 */
pub trait Blockchain {
    fn get_net();

    fn add_block(&self, block: impl Block) -> Result<impl Block, Box<dyn error::Error>>;

    fn get_block(&self, block_id: String) -> Result<impl Block, Box<dyn error::Error>>;
}
