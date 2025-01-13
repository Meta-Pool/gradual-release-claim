#!/bin/bash
__dir=$(dirname "$0")
. $__dir/0-mainnet-set-vars.sh

set -ex
# near view $CONTRACT_ADDRESS get_contract_info
# near view $CONTRACT_ADDRESS get_airdrops
# near view $CONTRACT_ADDRESS get_airdrops_by_status '{"status_code":0}'
# near view $CONTRACT_ADDRESS get_user_claims '{"account_id":"5e5657a757bb41d7cf7e3513124b02eea8d256649cff6f02311f4dfe5bab19ea"}'
near view $CONTRACT_ADDRESS get_user_claims '{"account_id":"0x16E44d8F1BEc8fb6c6770ABE8E8EF199B7723c9a.evmp.near"}'
near view $CONTRACT_ADDRESS get_users '{"from_index":0, "limit":150}'

