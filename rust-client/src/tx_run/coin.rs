use std::str::FromStr;

use anyhow::{Result, anyhow};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{TypeTag, StructTag};
use shared_crypto::intent::{IntentMessage, Intent};
use sui_sdk::SuiClient;
use sui_sdk::rpc_types::{SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions, SuiObjectResponseQuery, SuiObjectDataFilter};
use sui_sdk::types::base_types::{SuiAddress, ObjectRef, ObjectID};
use sui_sdk::types::crypto::{SuiKeyPair, Signature};
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_sdk::types::transaction::{ObjectArg, Command, Argument, TransactionData, Transaction};
use tracing::info;

use crate::gas::select_gas;

pub async fn get_treasury_cap(client: &SuiClient, owner_addr: SuiAddress, type_tag: TypeTag) -> Result<ObjectRef> {
    let resp = client
        .read_api()
        .get_owned_objects(
            owner_addr,
            Some(SuiObjectResponseQuery {
                filter: Some(SuiObjectDataFilter::StructType(StructTag {
                    address: AccountAddress::from_hex_literal("0x2")?,
                    module: Identifier::from_str("coin")?,
                    name: Identifier::from_str("TreasuryCap")?,
                    type_params: vec![type_tag],
                })),
                options: None,
            }),
            None,
            None,
        )
        .await?;

    let treasury_cap = resp
        .data
        .into_iter()
        .next()
        .ok_or(anyhow!("No deny-cap found!"))?;
    Ok(treasury_cap.data.ok_or(anyhow!("DenyCap empty!"))?.object_ref())
}

pub async fn mint_and_transfer(client: &SuiClient, signer: &SuiKeyPair, type_tag: TypeTag, treasury_cap: ObjectRef, to_address: SuiAddress, balance: u64) -> Result<SuiTransactionBlockResponse> {
    info!("MINTING COIN OF BALANCE {balance} TO ADDRESS {to_address}");
    let signer_addr = SuiAddress::from(&signer.public());
    let gas_data = select_gas(client, signer_addr, None, None, vec![], None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let treasury_cap = ptb.obj(ObjectArg::ImmOrOwnedObject(treasury_cap))?;
    let balance = ptb.pure(balance)?;
    ptb.command(Command::move_call(
        ObjectID::from_single_byte(0x2),
        Identifier::from_str("coin")?,
        Identifier::from_str("mint")?,
        vec![type_tag],
        vec![treasury_cap, balance],
    ));
    ptb.transfer_arg(to_address, Argument::Result(0));

    let builder = ptb.finish();

    // Sign transaction
    let msg = IntentMessage {
        intent: Intent::sui_transaction(),
        value: TransactionData::new_programmable(
            signer_addr,
            vec![gas_data.object],
            builder,
            gas_data.budget,
            gas_data.price,
        ),
    };
    let sig = Signature::new_secure(&msg, signer);

    let res = client
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(msg.value, vec![sig]),
            SuiTransactionBlockResponseOptions::new()
                .with_effects()
                .with_input(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    Ok(res)
}


