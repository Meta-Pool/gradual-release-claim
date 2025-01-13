#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-testnet-set-vars.sh

REQUIRED_ARGS=2
if [ $# -ne $REQUIRED_ARGS ]; then
  echo "Error: usage transfer-tokens-into-contract <token-account> <transfer-amount>"
  exit 1
fi
TOKEN_ADDRESS=$1
TRANSFER_AMOUNT=$2

. ./transfer-tokens-into.sh $CONTRACT_ADDRESS $TOKEN_ADDRESS $TRANSFER_AMOUNT
