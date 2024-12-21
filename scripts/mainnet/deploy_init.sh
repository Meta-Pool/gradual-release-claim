#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

echo DEPLOYING $NEAR_ENV META VOTE
set -ex
NEAR_ENV=mainnet near deploy $CONTRACT_ADDRESS $CONTRACT_WASM \
    --initFunction new --initArgs "$ARGS_INIT"

echo "remember to call token.storage_deposit(account_id=$CONTRACT_ADDRESS) so the contract can receive the airdrop tokens"
# NEAR_ENV=mainnet near call $MPDAO_TOKEN_ADDRESS storage_deposit '{"account_id":"'$METAVOTE_CONTRACT_ADDRESS'"}' --accountId $OWNER_ID --amount 0.0125