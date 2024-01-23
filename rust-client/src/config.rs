use move_core_types::language_storage::TypeTag;
use sui_sdk::wallet_context::WalletContext;
use sui_sdk::SuiClient;

pub struct AppConfig {
    pub client: SuiClient,
    pub wallet_context: WalletContext,
    pub type_tag: TypeTag,
}
