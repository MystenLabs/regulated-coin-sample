mod deny;

use std::str::FromStr;

use anyhow::{Result, anyhow};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};
use sui_sdk::SuiClient;
use sui_sdk::rpc_types::{SuiObjectDataOptions, SuiObjectResponseQuery, SuiObjectDataFilter, SuiTransactionBlockResponse};
use sui_sdk::types::SUI_DENY_LIST_OBJECT_ID;
use sui_sdk::types::base_types::{ObjectID, SequenceNumber, SuiAddress, ObjectRef};
use sui_sdk::types::object::Owner;

use crate::config::AppConfig;
use crate::command::AppCommand;

async fn get_deny_list(client: &SuiClient) -> Result<(ObjectID, SequenceNumber)> {
    let resp = client
        .read_api()
        .get_object_with_options(
            SUI_DENY_LIST_OBJECT_ID,
            SuiObjectDataOptions {
                show_type: true,
                show_owner: true,
                show_previous_transaction: false,
                show_display: false,
                show_content: false,
                show_bcs: false,
                show_storage_rebate: false,
            },
        )
        .await?;
    let deny_list = resp.data.ok_or(anyhow!("No deny-list found!"))?;
    let Some(Owner::Shared {
        initial_shared_version,
    }) = deny_list.owner
    else {
        return Err(anyhow!("Invalid deny-list owner!"));
    };
    Ok((SUI_DENY_LIST_OBJECT_ID, initial_shared_version))
}

async fn get_deny_cap(client: &SuiClient, owner_addr: SuiAddress, type_tag: TypeTag) -> Result<ObjectRef> {

    let resp = client
        .read_api()
        .get_owned_objects(
            owner_addr,
            Some(SuiObjectResponseQuery {
                filter: Some(SuiObjectDataFilter::StructType(StructTag {
                    address: AccountAddress::from_hex_literal("0x2")?,
                    module: Identifier::from_str("coin")?,
                    name: Identifier::from_str("DenyCap")?,
                    type_params: vec![type_tag],
                })),
                options: None,
            }),
            None,
            None,
        )
        .await?;

    let deny_cap = resp
        .data
        .into_iter()
        .next()
        .ok_or(anyhow!("No deny-cap found!"))?;
    Ok(deny_cap.data.ok_or(anyhow!("DenyCap empty!"))?.object_ref())
}

pub async fn execute_command(command: AppCommand, config: AppConfig) -> Result<SuiTransactionBlockResponse> {
    let AppConfig { client, mut wallet_context, type_tag } = config;

    match command {
        AppCommand::DenyListAdd(address) => {
            let deny_list = get_deny_list(&client).await?;
            let deny_cap = get_deny_cap(&client, wallet_context.active_address()?, type_tag.clone()).await?;
            deny::deny_list_add(&client, &mut wallet_context, type_tag, deny_list, deny_cap, address).await
        },
        AppCommand::DenyListRemove(address) => {
            let deny_list = get_deny_list(&client).await?;
            let deny_cap = get_deny_cap(&client, wallet_context.active_address()?, type_tag.clone()).await?;
            deny::deny_list_remove(&client, &mut wallet_context, type_tag, deny_list, deny_cap, address).await
        },
        _ => {todo!();}
    }
}
