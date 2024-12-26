#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-testnet-set-vars.sh

REQUIRED_ARGS=2
if [ $# -ne $REQUIRED_ARGS ]; then
  echo "Error: usage register-claims <airdrop-index> <token-account>"
  exit 1
fi

AIRDROP_INDEX=$1
TOKEN_ADDRESS=$2

near view $TOKEN_ADDRESS ft_metadata >temp.txt
cat temp.txt
SYMBOL=$(cat temp.txt | grep -oP 'symbol:.*'| cut -d':' -f2 | tr -d "," | xargs)
#remove the comma
SYMBOL=$(echo $SYMBOL | tr -d '"')
DECIMALS=$(cat temp.txt | grep -oP 'decimals:.*'| cut -d':' -f2 | xargs)
echo token:$SYMBOL, $DECIMALS decimals
rm temp.txt

# create a string of zeroes of length $DECIMALS
DECIMAL_ZEROES=$(printf "%0.s0" $(seq 1 $DECIMALS))

CLAIMS_SUM="250473147876" #5 decimals
DEC_MINUS_5=$(($DECIMALS - 5))
EXTRA_ZEROES=$(printf "%0.s0" $(seq 1 $DEC_MINUS_5))
CLAIMS_SUM_FULL="$CLAIMS_SUM$EXTRA_ZEROES"

# send tokens to the gradual release claims contract
near call $TOKEN_ADDRESS "ft_transfer" \
  '{"receiver_id":"'$CONTRACT_ADDRESS'","amount":"'$CLAIMS_SUM_FULL'"}' \
  --depositYocto 1 --accountId $OWNER_ID

# add claims distribution
# cspell:words silkking,lucastestmetavote,kuncho,agusin,andreatest,alnacklochnch
ADD_CLAIMS_ARGS=$(cat <<EOA
{
"airdrop_index":$AIRDROP_INDEX,
"total_amount":"$CLAIMS_SUM_FULL",
"data":[
["testwallet99.testnet","1200230"],
["silkking.testnet","400.54556"],
["lucastestmetavote.testnet","1300100"],
["lucio.testnet","200.2"],
["kuncho.testnet","300.3325"],
["agusin.testnet","600.15"],
["andreatest.testnet","700.0007"],
["alnacklochnch.testnet","800.25"],
["user-6.testnet","900"],
["user-7.testnet","500"]
]
}
EOA
)

near call $CONTRACT_ADDRESS "add_claims" "$ADD_CLAIMS_ARGS" --accountId $OPERATOR_ID --depositYocto  1

near call $CONTRACT_ADDRESS "enable_airdrop" '{"airdrop_index":'$AIRDROP_INDEX'}' --accountId $OPERATOR_ID --depositYocto  1

echo "remember to call $TOKEN_ADDRESS.storage_deposit(account_id:"xx") so the user can hold tokens"
# NEAR_ENV=testnet near call $MPDAO_TOKEN_ADDRESS storage_deposit '{"account_id":"'$METAVOTE_CONTRACT_ADDRESS'"}' --accountId $OWNER_ID --amount 0.0125
