// use crate::*;
// use near_sdk::{env, near_bindgen};

/*
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldState {
    pub owner_id: AccountId,
    pub operator_id: AccountId,
}

#[near_bindgen]
impl MetaVoteContract {
    #[init(ignore_state)]
    #[private] // only contract account can call this fn
    pub fn migrate() -> Self {
        // retrieve the current state from the contract
        let old: OldState = env::state_read().expect("failed");
        // return the new state
        Self {
            owner_id: old.owner_id,
            operator_id: old.operator_id,
        }
    }
}
*/