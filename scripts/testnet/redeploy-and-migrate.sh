#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-testnet-set-vars.sh

echo meta-vote-contract: $CONTRACT_ADDRESS
ls -l $CONTRACT_WASM

# re-Deploy and call state MIGRATION
echo RE-DEPLOY AND MIGRATION
set -ex
NEAR_ENV=testnet \
    near deploy $CONTRACT_ADDRESS $CONTRACT_WASM \
    --initFunction migrate --initArgs {}
