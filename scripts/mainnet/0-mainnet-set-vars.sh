#!/bin/bash
set -e
export NEAR_ENV="mainnet"

OWNER_ID="meta-pool-dao.near"
OPERATOR_ID="operator.mpdao-vote.near"
CONTRACT_ADDRESS="meta-pool-airdrop-gradual-release.near"
CONTRACT_WASM="res/gradual_release_claim_contract.wasm"

echo $NEAR_ENV $(date)

YOCTO_UNITS="000000000000000000000000"
TOTAL_PREPAID_GAS="300000000000000"

