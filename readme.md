# Regulated Coin Utility

#### Contains a sample move module and CLI tools to manage your Regulated Coin

### Available tools:

- Rust Tool: `/rust-client`
- TypeScript Tool: `/ts-client`

### Configuration

Rust Tool uses _$HOME/.sui/sui_config_ (`sui client` environment) while
Typescript Tool assumes that the following Environment Variables are set:

- `ADMIN_SECRET_KEY`: Admin's secret key for signing transactions.
- `SUI_FULLNODE_URL`: URL of the SUI network Node.

Both Tools need:

- `PACKAGE_ID`: The package of the regulated coin.
- `MODULE_NAME`: The module name that the regulated coin is created.


### Documentation

Details about **Coin** creation can be found in the Official Sui Docs [HERE](https://docs.sui.io/guides/developer/sui-101/create-coin)

Details about creation of a **Regulated Coin** can be found [HERE](https://docs.sui.io/guides/developer/sui-101/create-coin#create-regulated-coin).
