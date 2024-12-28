use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::{ext_contract, json_types::U128, log, PromiseResult};

use crate::*;

#[ext_contract(ext_self)]
#[allow(dead_code)]
pub trait ExtSelf {
    fn after_transfer_token(&mut self, account_id: &AccountId, airdrop_index: u16, amount: U128);
}

pub type VecUserClaims = Vec<UserClaimInfo>;

#[near_bindgen]
impl GradualReleaseContract {
    pub(crate) fn assert_only_owner(&self) {
        require!(
            self.owner_id == env::predecessor_account_id(),
            "Only the owner can call this function."
        );
    }
    pub(crate) fn assert_operator(&self) {
        require!(
            self.operator_id == env::predecessor_account_id(),
            "Only the operator can call this function."
        );
    }

    // internal method to get user claims or vec![].
    pub(crate) fn internal_get_claims(&self, account_id: &AccountId) -> VecUserClaims {
        self.available_claims
            .get(&account_id)
            .unwrap_or(VecUserClaims::new())
    }
    pub(crate) fn internal_get_claims_or_panic(&self, account_id: &AccountId) -> VecUserClaims {
        match self.available_claims.get(&account_id) {
            Some(a) => a,
            _ => panic!("{} has no claims", account_id),
        }
    }

    // distributes stNEAR or mpDAO between existent voters
    // called from ft_on_transfer
    pub(crate) fn internal_create_claims(
        &mut self,
        airdrop_index: u16,
        total_amount: u128,
        claims_array: Vec<(String, String)>,
    ) {
        let airdrop = &mut self.airdrops[airdrop_index as usize];
        assert!(
            airdrop.status_code == airdrop::status_code::DISABLED,
            "Airdrop {} is nor disabled. Can not add more claims",
            airdrop_index
        );
        let mut total_distributed = 0;
        for item in claims_array {
            let account_id = &AccountId::new_unchecked(item.0);
            let claims = &mut self
                .available_claims
                .get(account_id)
                .unwrap_or(VecUserClaims::new());

            if claims
                .into_iter()
                .any(|claim| claim.airdrop_index == airdrop_index)
            {
                panic!(
                    "{} already has a claim for airdrop {}",
                    account_id, airdrop_index
                );
            }
            let amount = parse_token_amount(&item.1, airdrop.token_decimals);
            claims.push(UserClaimInfo {
                airdrop_index,
                assigned_tokens: amount,
                claimed_tokens: 0,
            });
            // save
            self.available_claims.insert(&account_id, &claims);
            // sum total distributed
            total_distributed += amount;
        }

        assert!(
            total_distributed == total_amount,
            "total distributed {} != total_amount informed {}",
            total_distributed,
            total_amount
        );

        airdrop.total_distributed += total_distributed;

        // update total_in_claims UnorderedMap
        let current_amount = self
            .total_in_claims_per_token
            .get(&airdrop.token_contract)
            .unwrap_or(0);
        self.total_in_claims_per_token.insert(
            &airdrop.token_contract,
            &(current_amount + total_distributed),
        );
    }

    // before transfer
    pub(crate) fn remove_claimable_amount(
        &mut self,
        account_id: &AccountId,
        airdrop_index: u16,
    ) -> u128 {
        let user_claims = &mut self.internal_get_claims_or_panic(account_id);
        let airdrop = &mut self.airdrops[airdrop_index as usize];
        assert!(airdrop.is_enabled(), "Airdrop {} is not enabled", airdrop_index);
        let claim = match user_claims
            .iter_mut()
            .find(|i| i.airdrop_index == airdrop_index)
        {
            Some(c) => c, // claim is found
            None => panic!("{} has no claim for airdrop {}", account_id, airdrop_index),
        };
        let available_to_claim_now = claim.available_now(&airdrop.release_schedule);
        if available_to_claim_now == 0 {
            panic!(
                "0 available now. {} assigned:{} claimed:{}",
                airdrop.token_symbol, claim.assigned_tokens, claim.claimed_tokens,
            );
        };
        claim.claimed_tokens += available_to_claim_now;
        // save
        self.available_claims.insert(account_id, &user_claims);
        // update total claimed for the airdrop
        airdrop.total_claimed += available_to_claim_now;

        // remove from total in claims
        let current_amount = self
            .total_in_claims_per_token
            .get(&airdrop.token_contract)
            .unwrap_or(0);
        self.total_in_claims_per_token.insert(
            &airdrop.token_contract,
            &(current_amount - available_to_claim_now),
        );

        // return the amount
        available_to_claim_now
    }

    // rollback of the above fn
    pub(crate) fn re_add_claimable_amount(
        &mut self,
        account_id: &AccountId,
        airdrop_index: u16,
        amount: u128,
    ) {
        let user_claims = &mut self.internal_get_claims_or_panic(account_id);
        let airdrop = &mut self.airdrops[airdrop_index as usize];
        let claim = match user_claims
            .iter_mut()
            .find(|i| i.airdrop_index == airdrop_index)
        {
            Some(c) => c, // claim is found
            None => panic!("{} has no claim for airdrop {}", account_id, airdrop_index),
        };
        // restore
        claim.claimed_tokens -= amount;
        // save
        self.available_claims.insert(account_id, &user_claims);
        // undo total claimed sum for the airdrop
        airdrop.total_claimed -= amount;

        // re-add to total in claims
        let current_amount = self
            .total_in_claims_per_token
            .get(&airdrop.token_contract)
            .unwrap_or(0);
        self.total_in_claims_per_token.insert(
            &airdrop.token_contract,
            &(current_amount + &amount),
        );

    }

    pub(crate) fn internal_claim(&mut self, airdrop_index: u16, account_id: &AccountId) -> Promise {
        let amount = self.remove_claimable_amount(&account_id, airdrop_index);
        let airdrop = &self.airdrops[airdrop_index as usize];
        ext_ft_core::ext(airdrop.token_contract.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(
                account_id.clone(),
                U128::from(amount),
                Some(airdrop.title.clone()), // Memo
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_AFTER_TRANSFER)
                    .after_transfer_token(account_id, airdrop_index, U128::from(amount)),
            )
    }

    #[private]
    pub fn after_transfer_token(&mut self, account_id: &AccountId, airdrop_index: u16, amount: U128) {
        let airdrop = &self.airdrops[airdrop_index as usize];
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "{} claimed {} {} airdrop_index:{}",
                    account_id,
                    amount,
                    airdrop.token_symbol,
                    airdrop_index
                );
            }
            PromiseResult::Failed => {
                log!(
                    "FAIL: while claiming {} {} airdrop_index:{} user {}",
                    amount,
                    airdrop.token_symbol,
                    airdrop_index,
                    account_id,
                );
                // ROLLBACK
                self.re_add_claimable_amount(account_id, airdrop_index, amount);
            }
        };
    }
}
