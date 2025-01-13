#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-mainnet-set-vars.sh

echo meta-vote-contract: $CONTRACT_ADDRESS
ls -l $CONTRACT_WASM

# Redeploy Contract
echo Re-DEPLOY ONLY
NEAR_ENV=mainnet \
    near deploy $CONTRACT_ADDRESS $CONTRACT_WASM
