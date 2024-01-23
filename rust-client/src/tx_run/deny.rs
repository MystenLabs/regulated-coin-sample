use std::str::FromStr;

use anyhow::{Result, anyhow};
use move_core_types::identifier::Identifier;
use sui_sdk::SuiClient;
use sui_sdk::rpc_types::{SuiTransactionBlockResponseOptions, SuiTransactionBlockResponse};
use sui_sdk::types::TypeTag;
use sui_sdk::types::base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress};
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_sdk::types::transaction::{Command, ObjectArg, TransactionData};
use sui_sdk::wallet_context::WalletContext;

use crate::command::AppCommand;
use crate::gas::select_gas;

#[derive(Debug, Copy, Clone)]
pub enum DenyListCommand {
    Add(SuiAddress),
    Remove(SuiAddress)
}

impl TryFrom<AppCommand> for DenyListCommand {
    type Error = anyhow::Error;

    fn try_from(cmd: AppCommand) -> Result<Self> {
        match cmd {
            AppCommand::DenyListAdd(address) => Ok(DenyListCommand::Add(address)),
            AppCommand::DenyListRemove(address) => Ok(DenyListCommand::Remove(address)),
            _ => Err(anyhow!("Invalid command for deny list")),
        }
    }
}

impl DenyListCommand {
    pub fn address(&self) -> SuiAddress {
        match self {
            DenyListCommand::Add(addr) => *addr,
            DenyListCommand::Remove(addr) => *addr,
        }
    }
}

impl ToString for DenyListCommand {
    fn to_string(&self) -> String {
        match self {
            DenyListCommand::Add(_) => "deny_list_add",
            DenyListCommand::Remove(_) => "deny_list_remove",
        }.to_string()
    }
}

pub async fn deny_list_add(
    client: &SuiClient,
    wallet: &mut WalletContext,
    otw_type: TypeTag,
    deny_list: (ObjectID, SequenceNumber),
    deny_cap: ObjectRef,
    addr: SuiAddress,
) -> Result<SuiTransactionBlockResponse> {
    deny_list_cmd(client, wallet, DenyListCommand::Add(addr), otw_type, deny_list, deny_cap).await
}

pub async fn deny_list_remove(
    client: &SuiClient,
    wallet: &mut WalletContext,
    otw_type: TypeTag,
    deny_list: (ObjectID, SequenceNumber),
    deny_cap: ObjectRef,
    addr: SuiAddress,
) -> Result<SuiTransactionBlockResponse> {
    deny_list_cmd(client, wallet, DenyListCommand::Remove(addr), otw_type, deny_list, deny_cap).await
}

pub async fn deny_list_cmd(
    client: &SuiClient,
    wallet: &mut WalletContext,
    cmd: DenyListCommand,
    otw_type: TypeTag,
    deny_list: (ObjectID, SequenceNumber),
    deny_cap: ObjectRef,
) -> Result<SuiTransactionBlockResponse> {
    let signer_addr = wallet.active_address()?;
    let gas_data = select_gas(client, signer_addr, None, None, vec![], None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let deny_list = ptb.obj(ObjectArg::SharedObject {
        id: deny_list.0,
        initial_shared_version: deny_list.1,
        mutable: true,
    })?;
    let deny_cap = ptb.obj(ObjectArg::ImmOrOwnedObject(deny_cap))?;
    let address = ptb.pure(cmd.address())?;
    ptb.command(Command::move_call(
        ObjectID::from_single_byte(0x2),
        Identifier::from_str("coin")?,
        Identifier::from_str(&cmd.to_string())?,
        vec![otw_type],
        vec![deny_list, deny_cap, address],
    ));

    let builder = ptb.finish();

    let tx_data = TransactionData::new_programmable(
            signer_addr,
            vec![gas_data.object],
            builder,
            gas_data.budget,
            gas_data.price,
        );
    let tx = wallet.sign_transaction(&tx_data);

    let res = client
        .quorum_driver_api()
        .execute_transaction_block(
            tx,
            SuiTransactionBlockResponseOptions::new()
                .with_effects()
                .with_input(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    Ok(res)
}
