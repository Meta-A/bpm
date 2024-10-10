/**
 * Represents a block that is part of a Blockchain
 */
pub trait Block {
    fn get_id(&self) -> String;
    fn get_data(&self) -> String;
    fn get_signature(&self) -> String;
}
