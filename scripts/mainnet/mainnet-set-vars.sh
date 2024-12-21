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

# args to init meta-vote contract
        # owner_id: AccountId,
        # min_unbond_period: Days,
        # max_unbond_period: Days,
        # min_deposit_amount: U128String,
        # max_locking_positions: u8,
        # max_voting_positions: u8,
        # mpdao_token_contract_address: ContractAddress,
        # stnear_token_contract_address: ContractAddress,
        # registration_cost: U128String,
ARGS_INIT=$(cat <<EOA
{
"owner_id":"$OWNER_ID","operator_id":"$OPERATOR_ID"
}
EOA
)
