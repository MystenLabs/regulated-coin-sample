mod coin;
mod deny;

use anyhow::Result;
use sui_keys::keystore::AccountKeystore;
use sui_sdk::rpc_types::SuiTransactionBlockResponse;

use crate::command::AppCommand;
use crate::config::AppConfig;

pub async fn execute_command(
    command: AppCommand,
    config: AppConfig,
) -> Result<SuiTransactionBlockResponse> {
    let AppConfig {
        client,
        mut wallet_context,
        type_tag,
    } = config;
    let active_addr = wallet_context.active_address()?;
    let signer = wallet_context.config.keystore.get_key(&active_addr)?;

    match command {
        AppCommand::DenyListAdd(address) => {
            let deny_list = deny::get_deny_list(&client).await?;
            let deny_cap = deny::get_deny_cap(&client, active_addr, type_tag.clone()).await?;
            deny::deny_list_add(&client, signer, type_tag, deny_list, deny_cap, address).await
        }
        AppCommand::DenyListRemove(address) => {
            let deny_list = deny::get_deny_list(&client).await?;
            let deny_cap = deny::get_deny_cap(&client, active_addr, type_tag.clone()).await?;
            deny::deny_list_remove(&client, signer, type_tag, deny_list, deny_cap, address).await
        }
        AppCommand::MintAndTransfer(balance, to_address) => {
            let treasury_cap =
                coin::get_treasury_cap(&client, active_addr, type_tag.clone()).await?;
            coin::mint_and_transfer(&client, signer, type_tag, treasury_cap, to_address, balance)
                .await
        }
        AppCommand::Transfer(coin_id, to_address) => {
            let coin = coin::get_coin(&client, coin_id).await?;
            coin::transfer(&client, signer, coin, to_address).await
        }
        _ => {
            todo!();
        }
    }
}
