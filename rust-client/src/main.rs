use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};
use sui_config::{sui_config_dir, SUI_CLIENT_CONFIG};
use sui_sdk::rpc_types::{SuiObjectDataFilter, SuiObjectDataOptions, SuiObjectResponseQuery};
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::types::object::Owner;
use sui_sdk::types::SUI_DENY_LIST_OBJECT_ID;
use sui_sdk::wallet_context::WalletContext;
use tracing::debug;

use rust_client::deny::{deny_list_cmd, Contract, DenyListCommand};

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

struct Config {
    wallet_config: WalletContext,
    contract: Contract,
    command: DenyListCommand,
}

impl Config {
    /// Uses ~/.sui/config and Cli to construct
    async fn try_from_cli(cli: Cli) -> Result<Self> {
        let package_id_str = match cli.package_id {
            Some(package_id) => package_id,
            None => {
                dotenvy::dotenv().ok();
                std::env::var("PACKAGE_ID")?
            }
        };
        let package_id = ObjectID::from_hex_literal(&package_id_str)?;
        let otw = cli.module.to_uppercase();
        let contract = Contract {
            package_id,
            module: cli.module,
            otw,
        };
        let wallet_config = WalletContext::new(&sui_config_dir()?.join(SUI_CLIENT_CONFIG), None, None).await?;

        let command = match cli.command {
            CliCommand::Deny { address } => DenyListCommand::Add(SuiAddress::from_str(&address)?),
            CliCommand::Undeny { address } => {
                DenyListCommand::Remove(SuiAddress::from_str(&address)?)
            }
        };

        Ok(Config {
            wallet_config,
            contract,
            command,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let Config {
        mut wallet_config,
        contract,
        command,
    } = Config::try_from_cli(Cli::parse()).await?;
    let client = wallet_config.get_client().await?;

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
    let deny_list = (SUI_DENY_LIST_OBJECT_ID, initial_shared_version);

    let type_tag: TypeTag = (&contract).try_into()?;
    let resp = client
        .read_api()
        .get_owned_objects(
            wallet_config.active_address()?,
            Some(SuiObjectResponseQuery {
                filter: Some(SuiObjectDataFilter::StructType(StructTag {
                    address: AccountAddress::from_hex_literal("0x2")?,
                    module: Identifier::from_str("coin")?,
                    name: Identifier::from_str("DenyCap")?,
                    type_params: vec![type_tag.clone()],
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
        &client,
        &mut wallet_config,
        command,
        type_tag,
        deny_list,
        deny_cap,
    )
    .await?;

    debug!("{:?}", resp);

    Ok(())
}
