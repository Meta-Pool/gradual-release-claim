use crate::{constants::*, utils::*};
use near_contract_standards::fungible_token::{
    core::ext_ft_core,
    metadata::{ext_ft_metadata, FungibleTokenMetadata},
};
use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::unordered_map::UnorderedMap,
    env, ext_contract,
    json_types::{U128, U64},
    log, near_bindgen, require, AccountId, PanicOnDefault, Promise,
};
use user_claim_info::UserClaimInfo;

mod airdrop;
mod constants;
mod internal;
mod migrate;
mod user_claim_info;
mod utils;
mod view;

pub type Token = AccountId;

#[ext_contract(ext_self)]
#[allow(dead_code)]
trait ExtSelf {
    fn register_airdrop_step_2(
        &mut self,
        title: String,
        token_contract: AccountId,
        start_timestamp_ms: U64,
        end_timestamp_ms: U64,
    ) -> u16;

    fn enable_airdrop_step_2(&mut self, airdrop_index: u16);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
// Contract main state
pub struct GradualReleaseContract {
    pub owner_id: AccountId,
    pub operator_id: AccountId,

    // period is the same for everyone
    pub airdrops: Vec<airdrop::Airdrop>,

    pub available_claims: UnorderedMap<AccountId, Vec<UserClaimInfo>>, // claimable tokens per user
    pub total_in_claims_per_token: UnorderedMap<Token, u128>, // currently unclaimed -- increase on add_claims, decrease on claim
}

#[near_bindgen]
impl GradualReleaseContract {
    #[init]
    pub fn new(owner_id: AccountId, operator_id: AccountId) -> Self {
        require!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            operator_id,
            airdrops: vec![],
            available_claims: UnorderedMap::new(StorageKey::AvailableClaims),
            total_in_claims_per_token: UnorderedMap::new(StorageKey::TotalUnclaimed),
        }
    }

    // ***************
    // * owner config
    // ***************
    #[payable]
    pub fn set_operator_id(&mut self, operator_id: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.operator_id = operator_id;
    }
    #[payable]
    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.owner_id = owner_id;
    }

    #[payable]
    // timestamp in milliseconds
    // returns airdrop index
    pub fn register_airdrop(
        &mut self,
        title: String,
        token_contract: AccountId,
        start_timestamp_ms: U64,
        end_timestamp_ms: U64,
    ) -> Promise {
        self.assert_operator();
        assert_one_yocto();
        assert!(
            start_timestamp_ms.0 <= end_timestamp_ms.0,
            "Start timestamp_ms must be before end timestamp_ms"
        );
        // get token metadata to store token symbol and decimals
        ext_ft_metadata::ext(token_contract.clone())
            .with_static_gas(GAS_FOR_FT_METADATA)
            .ft_metadata()
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_REGISTER_AIRDROP_STEP_2)
                    .register_airdrop_step_2(
                        title,
                        token_contract,
                        start_timestamp_ms,
                        end_timestamp_ms,
                    ),
            )
    }
    // after obtaining token metadata
    #[private]
    pub fn register_airdrop_step_2(
        &mut self,
        title: String,
        token_contract: AccountId,
        start_timestamp_ms: U64,
        end_timestamp_ms: U64,
        #[callback] metadata: FungibleTokenMetadata,
    ) -> u16 {
        self.airdrops.push(airdrop::Airdrop {
            status_code: airdrop::status_code::DISABLED,
            title,
            token_contract,
            token_symbol: metadata.symbol,
            token_decimals: metadata.decimals,
            release_schedule: airdrop::TimestampPeriod {
                start_ms: start_timestamp_ms.0,
                end_ms: end_timestamp_ms.0,
            },
            total_distributed: 0,
            total_claimed: 0,
        });

        self.airdrops.len() as u16 - 1
    }

    // create claims for an inactive airdrop
    #[payable]
    pub fn add_claims(
        &mut self,
        airdrop_index: u16,
        total_amount: U128,
        data: Vec<(String, String)>,
    ) {
        assert_one_yocto();
        self.assert_operator();
        self.internal_create_claims(airdrop_index as u16, total_amount.0, data);
    }

    // after enough tokens have been transferred to the contract
    #[payable]
    pub fn enable_airdrop(&mut self, airdrop_index: u16) -> Promise {
        assert_one_yocto();
        self.assert_operator();
        let token_contract = self.airdrops[airdrop_index as usize].token_contract.clone();
        // get this contract's balance in the token
        // to ensure that the contract has enough tokens to distribute
        ext_ft_core::ext(token_contract)
            .ft_balance_of(env::current_account_id())
            .then(ext_self::ext(env::current_account_id()).enable_airdrop_step_2(airdrop_index))
    }
    #[private]
    pub fn enable_airdrop_step_2(
        &mut self,
        airdrop_index: u16,
        #[callback] contract_balance: U128,
    ) {
        let airdrop = &mut self.airdrops[airdrop_index as usize];
        let token_contract = airdrop.token_contract.clone();
        assert!(
            contract_balance.0 > 0,
            "ERR: This contract {} balance is 0 for token {}",
            env::current_account_id(),
            token_contract
        );

        airdrop.change_status(airdrop::status_code::ENABLED);

        let total_in_claims_this_token = self
            .total_in_claims_per_token
            .get(&token_contract)
            .unwrap_or(0);
        assert!(
            total_in_claims_this_token > 0,
            "ERR: for token {}, total_in_claims is 0",
            token_contract,
        );
        assert!(
            contract_balance.0 >= total_in_claims_this_token,
            "ERR: for token:{} contract_balance {} < total_in_claims {}",
            token_contract,
            contract_balance.0,
            total_in_claims_this_token
        );

        log!(
            "Airdrop index {} enabled for {} with contract_balance {} and total_in_claims {}",
            airdrop_index,
            token_contract,
            contract_balance.0,
            total_in_claims_this_token
        );
    }

    // archive an airdrop, no longer appears in get_airdrops()
    pub fn archive_airdrop(&mut self, airdrop_index: u16) {
        self.assert_operator();
        self.airdrops[airdrop_index as usize].change_status(airdrop::status_code::ARCHIVED);
    }

    // disable an airdrop, can be enabled later
    pub fn disable_airdrop(&mut self, airdrop_index: u16) {
        self.assert_operator();
        self.airdrops[airdrop_index as usize].change_status(airdrop::status_code::DISABLED);
    }

    // ------------------------
    // change airdrop schedule
    // ------------------------
    pub fn change_schedule(
        &mut self,
        airdrop_index: u16,
        start_timestamp_ms: U64,
        end_timestamp_ms: U64,
    ) {
        self.assert_operator();
        assert!(
            start_timestamp_ms.0 <= end_timestamp_ms.0,
            "Start timestamp_ms must be before end timestamp_ms"
        );
        self.airdrops[airdrop_index as usize].release_schedule = airdrop::TimestampPeriod {
            start_ms: start_timestamp_ms.0,
            end_ms: end_timestamp_ms.0,
        };
    }

    // ------------------------------------
    // user claims tokens
    // ------------------------------------
    pub fn claim(&mut self, airdrop_index: u16) -> Promise {
        self.internal_claim(airdrop_index, &env::predecessor_account_id())
    }

    // ------------------------------------
    // cleanup function, remove used claims
    // ------------------------------------
    pub fn remove_used_claims(&mut self, accounts: Vec<AccountId>) {
        for account_id in accounts {
            let user_claims_maybe = &mut self.available_claims.get(&account_id);
            if let Some(user_claims) = user_claims_maybe {
                user_claims.retain(|claim| claim.assigned_tokens > claim.claimed_tokens);
                // save
                if user_claims.is_empty() {
                    self.available_claims.remove(&account_id);
                } else {
                    self.available_claims.insert(&account_id, &user_claims);
                }
            }
        }
    }
}

// #[cfg(not(target_arch = "wasm32"))]
// #[cfg(test)]
// mod tests;
