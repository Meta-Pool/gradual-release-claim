use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    AccountId,
};

pub type StatusCode = u8;
pub mod status_code {
    pub const DISABLED: u8 = 0;
    pub const ENABLED: u8 = 1;
    pub const ARCHIVED: u8 = 2;
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TimestampPeriod {
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Airdrop {
    pub status_code: StatusCode,
    pub title: String,
    pub token_contract: AccountId,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub release_schedule: TimestampPeriod,
    pub total_distributed: u128,
    pub total_claimed: u128,
}

impl Airdrop {
    pub fn is_enabled(&self) -> bool {
        self.status_code == status_code::ENABLED
    }

    pub fn change_status(&mut self, new_status: StatusCode) {
        assert!(
            self.status_code != new_status,
            "ERR: Airdrop status is already {}",
            new_status
        );
        self.status_code = new_status;
    }
}
