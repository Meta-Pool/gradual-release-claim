# Gradual Release Claim Contract

This repository contains the Gradual Release Claim Contract

## Overview

The Gradual Release Claim Contract allows for the gradual release of tokens to a specified beneficiary over a predetermined period. This contract is designed to ensure a controlled and fair distribution of tokens.

## Features

- **Gradual Release**: Tokens are released gradually over time.
- **Beneficiary Management**: Specify and manage the beneficiary of the tokens.
- **Time-Based Release**: Configure the release schedule based on time intervals.

## Deployed contracts

    testnet: `meta-pool-airdrop-gradual-release.testnet`

## Usage (Front end)

1) given a connected user: call `get_user_claims{account_id:"user1.testnet"}`

2) you'll get a list of active airdrops for that user, you should create a component for each one, with a "claim" button.
The result is an array of: `Vec<ClaimInfoJSON>`
```rust
pub struct ClaimInfoJSON {
    /// true
    pub is_active: bool,

    /// 0..n
    pub airdrop_index: u16,

    /// title for the airdrop, e.g. "Meme Grant Round #9"
    pub airdrop_title: String,

    /// token to be claimed (contract)
    pub token_contract: AccountId,
    pub token_symbol: String,
    /// to display the amount correctly
    pub token_decimals: u8,

    /// total airdropped for this user
    pub assigned_tokens: U128,

    /// amount already claimed
    pub claimed_tokens: U128,

    /// available to claim NOW. this amount goes from zero to assigned_tokens during the gradual release
    /// this amount has subtracted already the "claimed_tokens" amount, and so, this amount will be reset
    /// to zero after a claim, and will increase on each minute boundary until release_end_ms
    pub available_tokens_now: U128,

    /// start of the gradual release period
    pub release_start_ms: U64,
    /// end of the gradual release period
    pub release_end_ms: U64,
}
```
3) If the user clicks `[CLAIM]`, you should:

3.1) verify is the user is registered *in the airdropped-token contract* (`token_contract.storage_balance_of(account_id:"user1")`),
     if not registered, call `token_contract.storage_deposit(account_id:"user1")` to allow the user to receive the tokens.
     If the user is not registered in the token, the claim will fail.

3.2) call `claim{airdrop_index:x}`, the tokens will be transferred to the user.

## Usage (admin)

1) Register an Airdrop event into the contract using `register_airdrop`, return value is the airdrop_index
2) call `add_claims(airdrop_id, amount, [["account_id","amount"],["account_id","amount"],...])` to distribute the tokens between any number of users
3) transfer the tokens to be distributed into the contract
4) call `enable_airdrop` to verify balances and enable the airdrop
5) each user can call `claim` during (and after) the release schedule

Note: It is important to call `storage_register` for the user (register the user with the token to be claimed)
before calling `claim` or the claim will fail

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
Go to metapool.app to find our discord, telegram and more documentation about Meta Pool

## License

This project is licensed under the MIT License.