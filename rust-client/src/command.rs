use sui_sdk::types::base_types::{SuiAddress, ObjectID};

#[derive(Debug)]
pub enum AppCommand {
    DenyListAdd(SuiAddress),
    DenyListRemove(SuiAddress),
    MintAndTransfer(u64, SuiAddress),
    Transfer(ObjectID, SuiAddress),
    Burn(ObjectID)
}

