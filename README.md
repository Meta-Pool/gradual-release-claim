# Gradual Release Claim Contract

This repository contains the Gradual Release Claim Contract

## Overview

The Gradual Release Claim Contract allows for the gradual release of tokens to a specified beneficiary over a predetermined period. This contract is designed to ensure a controlled and fair distribution of tokens.

## Features

- **Gradual Release**: Tokens are released gradually over time.
- **Beneficiary Management**: Specify and manage the beneficiary of the tokens.
- **Time-Based Release**: Configure the release schedule based on time intervals.


## Usage

1) Register an Airdrop event into the contract `register_airdrop`, return value is the airdrop_index
2) call `add_claims(airdrop_id, amount, [["account_id","amount"],["account_id","amount"],...])` to distribute the tokens between any number of users
3) transfer the tokens to be distributed into the contract
4) call `enable_airdrop` to verify balances and enable the airdrop

4) each user can call `claim` during (and after) the release schedule

Note: It is important to call `storage_register` for the user (register the user with the token to be claimed)
before calling `claim` or the claim will fail

## Configuration

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
Go to metapool.app to find our discord, telegram and more documentation about Meta Pool

## License

This project is licensed under the MIT License.