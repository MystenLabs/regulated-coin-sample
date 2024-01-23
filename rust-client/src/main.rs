use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{TypeTag, StructTag};
use sui_sdk::SuiClientBuilder;
use sui_sdk::rpc_types::{SuiObjectDataOptions, SuiObjectResponseQuery, SuiObjectDataFilter};
use sui_sdk::types::SUI_DENY_LIST_OBJECT_ID;
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::types::crypto::SuiKeyPair;
use sui_sdk::types::object::Owner;
use tracing::info;

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
    /// The secret key of the coin issuer. If none is passed, .env `ISSUER_SECRET_KEY` will be
    /// used. Lastly sui keystore file will be searched.
    #[arg(long = "secret-key", short = 's')]
    secret_key: Option<String>,
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
    // TODO signer of keystore
    signer: SuiKeyPair,
    contract: Contract,
    command: DenyListCommand,
}

impl TryFrom<Cli> for Config {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self> {
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
        let secret_key_str = match cli.secret_key {
            Some(secret_key) => Some(secret_key),
            None => {
                dotenvy::dotenv().ok();
                std::env::var("ISSUER_SECRET_KEY").ok()
            }
        };
        let signer: SuiKeyPair = match secret_key_str {
            Some(secret_key) => SuiKeyPair::from_str(&secret_key)
                .map_err(|e| anyhow!("Invalid secret key: {}", e))?,
            None => {
                todo!("Implement keystore signer")
            }
        };

        let command = match cli.command {
            CliCommand::Deny { address } => DenyListCommand::Add(SuiAddress::from_str(&address)?),
            CliCommand::Undeny { address } => {
                DenyListCommand::Remove(SuiAddress::from_str(&address)?)
            }
        };

        Ok(Config {
            signer,
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

    // TODO Support sui Config for url instead of var

    let config = Config::try_from(Cli::parse())?;
    let url = std::env::var("SUI_FULLNODE_URL")?;
    let client = SuiClientBuilder::default().build(url).await?;

    let resp = client.read_api().get_object_with_options(
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
    ).await?;

    let deny_list = resp.data.ok_or(anyhow!("No deny-list found!"))?;
    let Some(Owner::Shared { initial_shared_version }) = deny_list.owner else {
        return Err(anyhow!("Invalid deny-list owner!"));
    };
    let deny_list = (SUI_DENY_LIST_OBJECT_ID, initial_shared_version);

    let type_tag: TypeTag = (&config.contract).try_into()?;
    let resp = client.read_api().get_owned_objects(
        SuiAddress::from(&config.signer.public()),
        Some(SuiObjectResponseQuery {
            filter: Some(
                SuiObjectDataFilter::StructType(
                    StructTag {

                        address: AccountAddress::from_hex_literal("0x2")?,
                        module: Identifier::from_str("coin")?,
                        name: Identifier::from_str("DenyCap")?,
                        type_params: vec![type_tag.clone()]
                        }
                )),
            options: None }
            )
        , None, None).await?;

    let deny_cap = resp.data.into_iter().next().ok_or(anyhow!("No deny-cap found!"))?;
    let deny_cap = deny_cap.data.ok_or(anyhow!("DenyCap empty!"))?
        .object_ref();
    let resp = deny_list_cmd(
        &client,
        &config.signer,
        config.command,
        type_tag,
        deny_list,
        deny_cap,
    ).await?;

    info!("{:?}", resp);

    Ok(())
}
