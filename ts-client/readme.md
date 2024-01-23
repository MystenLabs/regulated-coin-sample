
# Stablecoin Utility CLI

CLI tool to manage your Stablecoin using the `@mysten/sui.js` library.

#### Options:
```shell
-V, --version                output the version number
-h, --help                   display help for command
````

#### Commands:
```
deny-list-add [options]      Adds an address to the deny list
deny-list-remove [options]   Removes an address from the deny list
mint-and-transfer [options]  mints coins and transfers to an address
burn [options]               mints coins and transfers to an address
help                         prints help
```


## Prerequisites

- Node.js
- npm


## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/MystenLabs/regulated-coin-sample.git
   cd stablecoin-utility
   ```

2. Install dependencies:

   ```bash
   npm install
   ```

## Configuration


You can use the provided move module `regulated_coin` as a base example for your own Regulated Coin, modify it to suit your needs and publish it to the SUI network

To publish the module and setup the nesessary environment variables, run the following commands:

```bash
# Make sure that you are at the root folder of the project.
./publish.sh
```

By default, this publishes the move module to the local network.

To publish to other networks, run :

```bash
./publish.sh testnet|devnet
````

The script will also set the necessary environment variables for you in the `.env` file.

```shell
#Demo .env file

SUI_FULLNODE_URL=http://localhost:9000
PACKAGE_ID=0x123...
DENY_CAP_ID=0x4556...
TREASURY_CAP_ID=0x789....
RUST_LOG=rust-client=DEBUG
```

If you don't use the provided publish script, make sure that you have the necessary environment variables set in a `.env` file or your environment:

- `ADMIN_SECRET_KEY`: Admin's secret key for signing transactions.
- `TREASURY_CAP_ID`: The Capability object that governs coin's economy!.
- `DENY_CAP_ID`: The capability object for the Deny List.
- `SUI_NETWORK`: URL of the SUI network.
- `MODULE_NAME`= The name of your published move module. eg. `regulated_coin`
- `COIN_NAME`= The name of the Coin. eg: `REGULATED_COIN`

## Usage

Run the CLI with the following commands:

### 1. Deny List Operations

#### Add Address to Deny List

```bash
npm run coin-manager -- deny-list-add --address <address>
or 
ts-node coin-manager deny-list-add --address <address>
```


#### Remove Address from Deny List

```bash
npm run coin-manager -- deny-list-remove --address <address> 
or
ts-node coin-manager deny-list-remove --address <address>
```

### 2. Mint and Transfer

```bash
npm run coin-manager -- mint-and-transfer --amount <amount> --address <address>
or 
ts-node coin-manager mint-and-transfer --amount <amount> --address <address>
```

### 3. Burn Coins

```bash
npm run coin-manager -- burn --coin <coinAddress>
or 
ts-node coin-manager burn --coin <coinAddress>
```

### 4. Help

```bash
npm run coin-manager help
or 
ts-node coin-manager help
```
