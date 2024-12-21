use near_sdk::{BorshStorageKey, Gas};
use near_sdk::borsh::{self, BorshSerialize};

pub const TGAS: u64 = 1_000_000_000_000;

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(50 * TGAS);
pub const GAS_FOR_AFTER_TRANSFER: Gas = Gas(40 * TGAS);
pub const GAS_FOR_FT_METADATA: Gas = Gas(5 * TGAS);
pub const GAS_FOR_REGISTER_AIRDROP_STEP_2: Gas = Gas(10 * TGAS);

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    AvailableClaims,
    TotalUnclaimed,
}

