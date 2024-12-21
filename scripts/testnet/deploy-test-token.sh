#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-testnet-set-vars.sh

REQUIRED_ARGS=3
if [ $# -ne $REQUIRED_ARGS ]; then
  echo "Error: usage deploy-test-token <account> <symbol> <decimals>"
  exit 1
fi

CONTRACT_WASM="res/nep141_test_token.wasm"

CONTRACT_ADDRESS=$1
SYMBOL=$2
DECIMALS=$3
# create a string of zeroes of length $DECIMALS
DECIMAL_ZEROES=$(printf "%0.s0" $(seq 1 $DECIMALS))

ONE_BILLION="1000000000"

ARGS_INIT=$(cat <<EOA
{
"owner_id":"$OWNER_ID",
"operator_id":"$OPERATOR_ID",
"symbol":"$SYMBOL",
"decimals":$DECIMALS,
"total_supply":"$ONE_BILLION$DECIMAL_ZEROES"
}
EOA
)

echo OWNER $OWNER_ID DEPLOYING $CONTRACT_ADDRESS $CONTRACT_WASM $SYMBOL $DECIMALS decimals 1BN initial supply
# wait for a key press
read -n 1 -s -r -p "Press any key to continue"

set -ex
NEAR_ENV=testnet near deploy $CONTRACT_ADDRESS $CONTRACT_WASM \
    --initFunction new_default_meta_2 --initArgs "$ARGS_INIT"

echo "remember to call $CONTRACT_ADDRESS.storage_deposit(account_id=) so the user can hold tokens"
# NEAR_ENV=testnet near call $MPDAO_TOKEN_ADDRESS storage_deposit '{"account_id":"'$METAVOTE_CONTRACT_ADDRESS'"}' --accountId $OWNER_ID --amount 0.0125
