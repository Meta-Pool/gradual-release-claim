use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

// ---- State structs -------

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserClaimInfo {
    pub airdrop_index: u16,
    pub assigned_tokens: u128,
    pub claimed_tokens: u128,
}

impl UserClaimInfo {
    /// Calculates the amount of tokens available for claim at the current block timestamp.
    ///
    /// # Arguments
    ///
    /// * `gradual_release_start_timestamp` - The timestamp when the gradual release period starts.
    /// * `gradual_release_end_timestamp` - The timestamp when the gradual release period ends.
    ///
    /// # Returns
    ///
    /// Returns the amount of tokens available for claim as a `u128`.
    ///
    /// # Logic
    ///
    /// - If the current timestamp is before the start of the gradual release period, returns 0.
    /// - If the current timestamp is after the end of the gradual release period, returns the difference between assigned tokens and claimed tokens.
    /// - If the current timestamp is within the gradual release period, calculates the proportion of tokens that should be unlocked based on the elapsed time and the total period length.
    /// - Returns the difference between the unlocked tokens and the claimed tokens, ensuring it does not return a negative value.
    pub fn available_now(&self, release_period: &airdrop::TimestampPeriod) -> u128 {
        let now_ms = get_current_epoch_millis();
        let unlocked_amount = if now_ms < release_period.start_ms {
            0
        } else if now_ms > release_period.end_ms {
            // gradual period is over
            self.assigned_tokens
        } else {
            // in the middle of the gradual release period
            let period_length_minutes = (release_period.end_ms - release_period.start_ms) / 60000;
            // we use minutes so extra tokens become available on each minute mark
            let elapsed_minutes = (now_ms - release_period.start_ms) / 60000;
            proportional(
                self.assigned_tokens,
                elapsed_minutes as u128,
                period_length_minutes as u128,
            )
        };
        unlocked_amount.saturating_sub(self.claimed_tokens)
    }

    pub fn is_active(&self) -> bool {
        self.assigned_tokens > 0 && self.claimed_tokens < self.assigned_tokens
    }
}
