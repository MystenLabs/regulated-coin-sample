#!/bin/bash

# check dependencies are available.
for i in jq curl sui; do
  if ! command -V ${i} 2>/dev/null; then
    echo "${i} is not installed"
    exit 1
  fi
done

NETWORK=http://localhost:9000
BACKEND_API=http://localhost:3000
FAUCET=https://localhost:9000/gas

# Put the dependant package, as the depending will be published too via --with-unpublished-dependencies
MOVE_PACKAGE_PATH=../move/regulated_coin_example

if [ $# -ne 0 ]; then
  if [ $1 = "testnet" ]; then
    NETWORK="https://rpc.testnet.sui.io:443"
    FAUCET="https://faucet.testnet.sui.io/gas"
    BACKEND_API="https://api-testnet.suifrens.sui.io"
  fi
  if [ $1 = "devnet" ]; then
    NETWORK="https://rpc.devnet.sui.io:443"
    FAUCET="https://faucet.devnet.sui.io/gas"
    BACKEND_API="https://api-devnet.suifrens.sui.io"
  fi
fi

# Change `ADMIN_NAME` to a variable for which you have the below:
# eg for USER1 as below:
# USER1_ADDRESS
# USER1_SECRET_KEY
# USER1_KEY_SCHEME
ADMIN_NAME=USER1
ADMIN_ADDRESS_NAME=${ADMIN_NAME}_ADDRESS
echo "- Publisher Address is: ${!ADMIN_ADDRESS_NAME}"

switch_res=$(sui client switch --address ${!ADMIN_ADDRESS_NAME})

publish_res=$(sui client publish --skip-fetch-latest-git-deps --gas-budget 2000000000 --json ${MOVE_PACKAGE_PATH})

echo ${publish_res} >.publish.res.json

# Check if the command succeeded (exit status 0)
if [[ "$publish_res" =~ "error" ]]; then
  # If yes, print the error message and exit the script
  echo "Error during move contract publishing.  Details : $publish_res"
  exit 1
fi

publishedObjs=$(echo "$publish_res" | jq -r '.objectChanges[] | select(.type == "published")')
PACKAGE_ID=$(echo "$publishedObjs" | jq -r '.packageId')

newObjs=$(echo "$publish_res" | jq -r '.objectChanges[] | select(.type == "created")')
DENY_CAP_ID=$(echo "$newObjs" | jq -r 'select(.objectType | contains("::coin::DenyCap<")).objectId')
TREASURY_CAP_ID=$(echo "$newObjs" | jq -r 'select(.objectType | contains("::coin::TreasuryCap<")).objectId')

suffix=""
if [ $# -eq 0 ]; then
  suffix=".localnet"
fi

# ADMIN_CAP_ID=$ADMIN_CAP_ID
cat >.env<<-API_ENV
SUI_FULLNODE_URL=$NETWORK
BACKEND_API=$BACKEND_API
PACKAGE_ID=$PACKAGE_ID
ADMIN_NAME=$ADMIN_NAME
DENY_CAP_ID=$DENY_CAP_ID
TREASURY_CAP_ID=$TREASURY_CAP_ID
RUST_LOG=rust-client=DEBUG


API_ENV

echo "Contract Deployment finished!"
