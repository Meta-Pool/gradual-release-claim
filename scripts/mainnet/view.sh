#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

set -ex
near view $CONTRACT_ADDRESS get_contract_info
near view $CONTRACT_ADDRESS get_airdrops
