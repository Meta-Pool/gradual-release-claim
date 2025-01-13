#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-testnet-set-vars.sh

REQUIRED_ARGS=2
if [ $# -ne $REQUIRED_ARGS ]; then
  echo "Error: usage alter-airdrop-title <airdrop-index> <new-title>"
  exit 1
fi
AIRDROP_INDEX=$1
NEW_TITLE=$2

set -ex
ALTER_ARGS=$(cat <<EOA
{
"airdrop_index":$AIRDROP_INDEX,
"new_title":"$NEW_TITLE"
}
EOA
)
echo "$ALTER_ARGS"

near call $CONTRACT_ADDRESS "alter_airdrop_title" "$ALTER_ARGS" --accountId $OPERATOR_ID

near call $TOKEN_ADDRESS "storage_deposit" '{"account_id":"'$CONTRACT_ADDRESS'"}' --accountId $OPERATOR_ID --deposit 0.0125

# create a string of zeroes of length $DECIMALS
DECIMAL_ZEROES=$(printf "%0.s0" $(seq 1 $DECIMALS))

# send tokens to the gradual release claims contract
near call $TOKEN_ADDRESS "ft_transfer" \
  '{"receiver_id":"'$CONTRACT_ADDRESS'","amount":"'$TRANSFER_AMOUNT$DECIMAL_ZEROES'"}' \
  --depositYocto 1 --accountId $OWNER_ID

