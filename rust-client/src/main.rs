use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};
use rust_client::command::AppCommand;
use rust_client::config::AppConfig;
use sui_config::{sui_config_dir, SUI_CLIENT_CONFIG};
use sui_sdk::rpc_types::{SuiObjectDataFilter, SuiObjectDataOptions, SuiObjectResponseQuery};
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::types::object::Owner;
use sui_sdk::types::SUI_DENY_LIST_OBJECT_ID;
use sui_sdk::wallet_context::WalletContext;
use tracing::debug;

use rust_client::deny::deny_list_cmd;

/// Regulated coin command line interface
#[derive(Parser, Debug)]
#[command(name = "rust-client")]
struct Cli {
    /// The address of the contract the coin is issued. If none is passed, .env `PACKAGE_ID` will be used.
    #[arg(long = "package-id", short = 'p')]
    package_id: Option<String>,
    /// The module that issues the coin.
    #[arg(long = "module", short = 'm', default_value = "regulated_coin")]
    module: String,
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Add an address to allow-list
    #[command(name = "deny")]
    Deny {
        /// The address to insert to deny-list
        #[arg(value_parser)]
        address: String,
    },
    /// Remove an address from deny-list
    #[clap(name = "undeny")]
    Undeny {
        /// The address to remove from deny-list
        #[arg(value_parser)]
        address: String,
    },
}

async fn cli_parse() -> Result<(AppConfig, AppCommand)> {
    let Cli {
        package_id,
        module,
        command,
    } = Cli::parse();
    let package_id_str = match package_id {
        Some(package_id) => package_id,
        None => {
            dotenvy::dotenv().ok();
            std::env::var("PACKAGE_ID")?
        }
    };
    let package_id = ObjectID::from_hex_literal(&package_id_str)?;
    let otw = module.to_uppercase();
    let type_tag = TypeTag::Struct(Box::new(StructTag {
        address: AccountAddress::new(package_id.as_ref().try_into()?),
        module: Identifier::from_str(&module)?,
        name: Identifier::from_str(&otw)?,
        type_params: vec![],
    }));
    let wallet_context =
        WalletContext::new(&sui_config_dir()?.join(SUI_CLIENT_CONFIG), None, None).await?;

    let command = match command {
        CliCommand::Deny { address } => AppCommand::DenyListAdd(SuiAddress::from_str(&address)?),
        CliCommand::Undeny { address } => AppCommand::DenyListRemove(SuiAddress::from_str(&address)?),

    };

    let client = wallet_context.get_client().await?;
    Ok((
        AppConfig {
            client,
            wallet_context,
            type_tag,
        },
        command,
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let (mut config, command) = cli_parse().await?;


    let resp = config.client
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
    let deny_list = (SUI_DENY_LIST_OBJECT_ID, initial_shared_version);

    let resp = config.client
        .read_api()
        .get_owned_objects(
            config.wallet_context.active_address()?,
            Some(SuiObjectResponseQuery {
                filter: Some(SuiObjectDataFilter::StructType(StructTag {
                    address: AccountAddress::from_hex_literal("0x2")?,
                    module: Identifier::from_str("coin")?,
                    name: Identifier::from_str("DenyCap")?,
                    type_params: vec![config.type_tag.clone()],
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
    let deny_cap = deny_cap.data.ok_or(anyhow!("DenyCap empty!"))?.object_ref();
    let resp = deny_list_cmd(
        &config.client,
        &mut config.wallet_context,
        command.try_into()?,
        config.type_tag,
        deny_list,
        deny_cap,
    )
    .await?;

    debug!("{:?}", resp);

    Ok(())
}
