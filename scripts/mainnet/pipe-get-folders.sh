set -ex
PIPE_CONTRACT_ADDRESS="meta-pipeline.near"
NEAR_ENV=mainnet near view $PIPE_CONTRACT_ADDRESS get_folders

