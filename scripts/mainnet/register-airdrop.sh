#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-mainnet-set-vars.sh

# get the timestamp in milliseconds
# current_timestamp_ms=$(date +%s%3N)
# # start is current timestamp
# start_timestamp_ms=$(($current_timestamp_ms + 0*60*60*1000))
# # end is current timestamp + 4 days
# end_timestamp_ms=$(($current_timestamp_ms + 4*24*60*60*1000))

# convert a iso date and time to unix timestamp in milliseconds
start_timestamp_ms=$(date -d "2024-12-30T15:00:00Z" +%s%3N)
end_timestamp_ms=$(date -d "2025-01-03T15:00:00Z" +%s%3N)
echo $start_timestamp_ms to $end_timestamp_ms

# check if the required arguments are passed
REQUIRED_ARGS=3
if [ $# -ne $REQUIRED_ARGS ]; then
  echo "Error: usage register-airdrop <airdrop-index> <grant-round-number> <token-account>"
  exit 1
fi
AIRDROP_INDEX=$1
ROUND_NUMBER=$2
TOKEN_ADDRESS=$3
TRANSFER_AMOUNT=$4

near view $TOKEN_ADDRESS ft_metadata >temp.txt
cat temp.txt
SYMBOL=$(cat temp.txt | grep -oP 'symbol:.*'| cut -d':' -f2 | tr -d "," | xargs)
#remove the comma
SYMBOL=$(echo $SYMBOL | tr -d '"')
DECIMALS=$(cat temp.txt | grep -oP 'decimals:.*'| cut -d':' -f2 | xargs)
echo token:$SYMBOL, $DECIMALS decimals
rm temp.txt

set -ex
REGISTER_ARGS=$(cat <<EOA
{
"title":"Grants #$ROUND_NUMBER - $SYMBOL Airdrop for voters",
"token_contract":"$TOKEN_ADDRESS",
"start_timestamp_ms":"$start_timestamp_ms",
"end_timestamp_ms":"$end_timestamp_ms"
}
EOA
)
echo "$REGISTER_ARGS"

near call $CONTRACT_ADDRESS "register_airdrop" "$REGISTER_ARGS" --accountId $OPERATOR_ID --depositYocto 1

#near call $TOKEN_ADDRESS "storage_deposit" '{"account_id":"'$CONTRACT_ADDRESS'"}' --accountId $OPERATOR_ID --deposit 0.0125

# create a string of zeroes of length $DECIMALS
#DECIMAL_ZEROES=$(printf "%0.s0" $(seq 1 $DECIMALS))

# send tokens to the gradual release claims contract
#near call $TOKEN_ADDRESS "ft_transfer" \
  # '{"receiver_id":"'$CONTRACT_ADDRESS'","amount":"'$TRANSFER_AMOUNT$DECIMAL_ZEROES'"}' \
  # --depositYocto 1 --accountId $OWNER_ID

