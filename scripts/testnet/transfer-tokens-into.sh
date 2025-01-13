#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-testnet-set-vars.sh

REQUIRED_ARGS=3
if [ $# -ne $REQUIRED_ARGS ]; then
  echo "Error: usage transfer-tokens-into <receiver> <token-account> <transfer-amount>"
  exit 1
fi

RECEIVER=$1
TOKEN_ADDRESS=$2
TRANSFER_AMOUNT=$3

near view $TOKEN_ADDRESS ft_metadata >temp.txt
cat temp.txt
SYMBOL=$(cat temp.txt | grep -oP 'symbol:.*'| cut -d':' -f2 | tr -d "," | xargs)
#remove the comma
SYMBOL=$(echo $SYMBOL | tr -d '"')
DECIMALS=$(cat temp.txt | grep -oP 'decimals:.*'| cut -d':' -f2 | xargs)
echo token:$SYMBOL, $DECIMALS decimals
rm temp.txt

near call $TOKEN_ADDRESS "storage_deposit" '{"account_id":"'$RECEIVER'"}' --accountId $OPERATOR_ID --deposit 0.0125

# create a string of zeroes of length $DECIMALS
DECIMAL_ZEROES=$(printf "%0.s0" $(seq 1 $DECIMALS))

# send tokens to the gradual release claims contract
near call $TOKEN_ADDRESS "ft_transfer" \
  '{"receiver_id":"'$RECEIVER'","amount":"'$TRANSFER_AMOUNT$DECIMAL_ZEROES'"}' \
  --depositYocto 1 --accountId $OWNER_ID

