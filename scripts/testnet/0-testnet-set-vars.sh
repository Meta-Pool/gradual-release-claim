#!/bin/bash
set -e
export NEAR_ENV="testnet"

OWNER_ID="mpdao-vote.testnet"
OPERATOR_ID="operator.mpdao-vote.testnet"
CONTRACT_ADDRESS="meta-pool-airdrop-gradual-release.testnet"
CONTRACT_WASM="res/gradual_release_claim_contract.wasm"

echo $NEAR_ENV $(date)

YOCTO_UNITS="000000000000000000000000"
TOTAL_PREPAID_GAS="300000000000000"

# args to init contract
ARGS_INIT=$(cat <<EOA
{
"owner_id":"$OWNER_ID",
"operator_id":"$OPERATOR_ID"
}
EOA
)
