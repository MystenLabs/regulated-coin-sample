# Usage

The below commands are signed using the sui client's wallet active address and environment.

#### `rust-client --help`
```
Usage: rust-client [OPTIONS] <COMMAND>

Commands:
  deny-list-add      Add an address to allow-list
  deny-list-remove   Remove an address from deny-list
  mint-and-transfer  Mint and transfer coin
  transfer           Transfer coin from the sui client's active address
  burn               Burn coin inside the sui client's active address
  help               Print this message or the help of the given subcommand(s)

Options:
  -p, --package-id <PACKAGE_ID>  The address of the contract the coin is issued. If none is passed, .env `PACKAGE_ID` will be used
  -m, --module <MODULE>          The module that issues the coin [default: regulated_coin]
  -h, --help                     Print help
```

### Examples

0xf6d34bf1bb4243a7250da5b16add57c87214ad6be10a9ebb35dadeb5915e9b31 is a Sui address.

0x22048e8de5f1669d4f058efb0b05c7f401aeb59993e6d66600fdafe53a86ebf8 is a Coin object-id.

- `rust-client deny-list-add 0xf6d34bf1bb4243a7250da5b16add57c87214ad6be10a9ebb35dadeb5915e9b31`
- `rust-client deny-list-remove 0xf6d34bf1bb4243a7250da5b16add57c87214ad6be10a9ebb35dadeb5915e9b31`
- `rust-client mint-and-transfer -b 10000 0xf6d34bf1bb4243a7250da5b16add57c87214ad6be10a9ebb35dadeb5915e9b31`
- `rust-client transfer -c 0x22048e8de5f1669d4f058efb0b05c7f401aeb59993e6d66600fdafe53a86ebf8 0xf6d34bf1bb4243a7250da5b16add57c87214ad6be10a9ebb35dadeb5915e9b31`
- `rust-client burn 0x22048e8de5f1669d4f058efb0b05c7f401aeb59993e6d66600fdafe53a86ebf8`

