use near_sdk::{
    json_types::{U128, U64},
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::*;

// ---------------------
//   View functions
// ---------------------
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractInfoJSON {
    pub owner_id: String,
    pub operator_id: String,
    pub airdrop_count: u16,
    pub user_count: u64,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AirdropJSON {
    pub airdrop_index: u16,
    pub enabled: bool,
    pub title: String,
    pub token_contract: AccountId,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub release_schedule_start_ms: U64,
    pub release_schedule_end_ms: U64,
    pub total_distributed: U128,
    pub total_claimed: U128,
}

// ---- as JSON to return from view calls ---
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ClaimInfoJSON {
    pub is_active: bool,
    pub airdrop_index: u16,
    pub airdrop_title: String,
    pub token_contract: AccountId,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub assigned_tokens: U128,
    pub claimed_tokens: U128,
    pub available_tokens_now: U128,
    pub release_start_ms: U64,
    pub release_end_ms: U64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct UserClaimsJSON {
    pub account_id: AccountId,
    pub claims: Vec<ClaimInfoJSON>,
}

#[near_bindgen]
impl GradualReleaseContract {
    pub fn get_owner_id(&self) -> String {
        self.owner_id.to_string()
    }
    pub fn get_operator_id(&self) -> String {
        self.operator_id.to_string()
    }

    pub fn get_contract_info(&self) -> ContractInfoJSON {
        ContractInfoJSON {
            owner_id: self.owner_id.as_str().into(),
            operator_id: self.operator_id.as_str().into(),
            airdrop_count: self.airdrops.len() as u16,
            user_count: self.available_claims.len(),
        }
    }

    pub fn get_total_in_claims_per_token(self, token_contract: AccountId) -> U128 {
        self.total_in_claims_per_token
            .get(&token_contract)
            .unwrap_or(0)
            .into()
    }

    pub fn get_airdrops(&self) -> Vec<AirdropJSON> {
        self.internal_get_airdrops(false)
    }
    pub fn get_airdrops_including_not_enabled(&self) -> Vec<AirdropJSON> {
        self.internal_get_airdrops(true)
    }

    pub(crate) fn internal_get_airdrops(&self, include_disabled: bool) -> Vec<AirdropJSON> {
        self.airdrops
            .iter()
            .enumerate()
            .filter(|(_, a)| include_disabled || a.enabled)
            .map(|(index, a)| AirdropJSON {
                airdrop_index: index as u16,
                enabled: a.enabled,
                title: a.title.clone(),
                token_contract: a.token_contract.clone(),
                token_symbol: a.token_symbol.clone(),
                token_decimals: a.token_decimals,
                release_schedule_start_ms: a.release_schedule.start_ms.into(),
                release_schedule_end_ms: a.release_schedule.end_ms.into(),
                total_distributed: U128(a.total_distributed),
                total_claimed: U128(a.total_claimed),
            })
            .collect()
    }

    // get all information for a single voter: voter + locking-positions + voting-positions
    pub fn get_user_claims(&self, account_id: &AccountId) -> Vec<ClaimInfoJSON> {
        self.claims_to_json(self.internal_get_claims(&account_id).into_iter(), false)
    }

    // get all information for a single voter: voter + locking-positions + voting-positions
    pub fn get_user_claims_including_inactive(&self, account_id: &AccountId) -> Vec<ClaimInfoJSON> {
        self.claims_to_json(self.internal_get_claims(&account_id).into_iter(), true)
    }

    // get all information for multiple voters, by index: Vec<voter + locking-positions + voting-positions>
    pub fn get_users(&self, from_index: u32, limit: u32) -> Vec<UserClaimsJSON> {
        let keys = self.available_claims.keys_as_vector();
        let voters_len = keys.len() as u64;
        let start = from_index as u64;
        let limit = limit as u64;

        let mut results = Vec::<UserClaimsJSON>::new();
        for index in start..std::cmp::min(start + limit, voters_len) {
            let account_id = keys.get(index).unwrap();
            let claims = self.available_claims.get(&account_id).unwrap();
            results.push(UserClaimsJSON {
                account_id: account_id.clone(),
                claims: self.claims_to_json(claims.into_iter(), true),
            });
        }
        results
    }

    pub(crate) fn claims_to_json<I>(&self, iter: I, include_inactive: bool) -> Vec<ClaimInfoJSON>
    where
        I: Iterator<Item = UserClaimInfo>,
    {
        let mut result = Vec::new();
        for claim in iter {
            let airdrop = &self.airdrops[claim.airdrop_index as usize];
            if airdrop.enabled && (include_inactive || claim.is_active()) {
                result.push(ClaimInfoJSON {
                    is_active: claim.is_active(),
                    airdrop_index: claim.airdrop_index,
                    airdrop_title: airdrop.title.clone(),
                    token_symbol: airdrop.token_symbol.clone(),
                    token_decimals: airdrop.token_decimals,
                    token_contract: airdrop.token_contract.clone(),
                    assigned_tokens: U128(claim.assigned_tokens),
                    claimed_tokens: U128(claim.claimed_tokens),
                    available_tokens_now: claim.available_now(&airdrop.release_schedule).into(),
                    release_start_ms: airdrop.release_schedule.start_ms.into(),
                    release_end_ms: airdrop.release_schedule.end_ms.into(),
                })
            };
        }
        result
    }
}
