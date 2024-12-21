use futures::future::join_all;
use near_sdk::json_types::{U128, U64};
use tokio::join;

use core::str;
use near_gas::*;
use near_workspaces::{
    network::Sandbox,
    types::{KeyType, NearToken, SecretKey},
    Account, AccountId, Contract, DevNetwork, Worker,
};
use std::{collections::HashMap, str::FromStr};

mod nep141_test_utils;
mod test_utils;

use nep141_test_utils::*;
use test_utils::*;

const GRADUAL_RELEASE_CONTRACT_FILEPATH: &str = "res/gradual_release_claim_contract.wasm";

pub const E6: u128 = 1_000_000;
pub const E24: u128 = 1_000_000_000_000_000_000_000_000;

pub const ONE_TEST_TOKEN_A: u128 = 1_000_000;
pub const ONE_TEST_TOKEN_B: u128 = 1_000_000_000_000;

pub(crate) const DEV_ACCOUNT_SEED: &str = "testificate";

pub async fn create_account(worker: &Worker<Sandbox>, name: &str) -> Account {
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    //    let (id, sk) = self.dev_generate().await;
    let account_id = AccountId::from_str(&format!("{}", name)).unwrap();
    worker.create_tla(account_id, sk).await.unwrap().unwrap()
}

async fn create_gradual_release_contract(
    worker: &Worker<impl DevNetwork>,
    name: &str,
    owner: &Account,
    operator: &Account,
) -> Contract {
    let wasm_code = std::fs::read(GRADUAL_RELEASE_CONTRACT_FILEPATH).unwrap();
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let account_id = AccountId::from_str(&format!("{}", name)).unwrap();
    let gradual_release_contract = worker
        .create_tla_and_deploy(account_id, sk, &wasm_code)
        .await
        .unwrap()
        .unwrap();
    // initialize the contract
    exec(
        owner
            .call(&gradual_release_contract.id(), "new")
            .args_json(serde_json::json!({
                "owner_id": owner.id(),
                "operator_id": operator.id(),
            })),
    )
    .await;
    gradual_release_contract
}

#[derive(Debug)]
pub struct TokenInfo {
    contract_account_id: AccountId,
    symbol: String,
    decimals: u8,
    total_supply: u64,
    airdrop_amount: u64,
    airdrop_index: u16,
}

impl TokenInfo {
    pub fn amount_from_string_dec(&self, amount_string_dec: &String) -> u128 {
        parse_token_amount(amount_string_dec, self.decimals)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("init sandbox");
    let worker = near_workspaces::sandbox().await?;

    // Creating Accounts & tokens
    println!("Creating Accounts");
    let (owner, operator, user_zero, user_one, user_two) = join!(
        create_account(&worker, "owner"),
        create_account(&worker, "operator"),
        create_account(&worker, "zero"),
        create_account(&worker, "one"),
        create_account(&worker, "two"),
    );
    let users = vec![user_zero, user_one, user_two];
    println!("Owner: {}", &owner.id());
    println!("Operator: {}", &operator.id());
    println!("user_zero: {}", &users[0].id());

    let gradual_release_contract = create_gradual_release_contract(
        &worker,
        "gradual_release_claim_contract",
        &owner,
        &operator,
    )
    .await;
    println!(
        "gradual_release_contract: {}",
        gradual_release_contract.id()
    );

    // create nep141 tokens to airdrop
    let tokens = vec![
        TokenInfo {
            contract_account_id: AccountId::from_str("token-air-0").unwrap(),
            symbol: "AIR-0".to_string(),
            decimals: 6,
            total_supply: 200_000_000,
            airdrop_amount: 1_520_000,
            airdrop_index: 0,
        },
        TokenInfo {
            contract_account_id: AccountId::from_str("token-air-1").unwrap(),
            symbol: "AIR-1".to_string(),
            decimals: 16,
            total_supply: 500_000_000,
            airdrop_amount: 3_000_000,
            airdrop_index: 1,
        },
    ];

    let nep141_contracts = join_all(tokens.iter().map(|info| {
        create_nep141_token(
            &worker,
            &owner,
            &info.contract_account_id,
            &info.symbol,
            info.decimals,
            info.total_supply as u128 * 10u128.pow(info.decimals as u32),
        )
    }))
    .await;

    let token_accounts: Vec<AccountId> = tokens
        .iter()
        .map(|info| info.contract_account_id.clone())
        .collect();
    let mut holders = users.iter().skip(1).collect::<Vec<&Account>>().clone();
    let contract_as_holder = gradual_release_contract.as_account();
    holders.push(contract_as_holder);
    register_storage_cartesian(&token_accounts, &holders).await;

    // send the tokens to the airdrop contract
    join_all(tokens.iter().map(|info| {
        ft_transfer(
            &info.contract_account_id,
            &owner,
            &gradual_release_contract.as_account(),
            info.airdrop_amount as u128 * 10u128.pow(info.decimals as u32),
        )
    }))
    .await;

    // register the airdrops in the contract
    let current_timestamp_ms = chrono::Utc::now().timestamp_millis() as u64;
    for info in tokens.iter() {
        let args = serde_json::json!({
            "title": format!("Airdrop of {}",info.symbol),
            "token_contract": info.contract_account_id,
            "start_timestamp_ms": U64(current_timestamp_ms + 20_000),
            "end_timestamp_ms": U64(current_timestamp_ms + 30_000),
        });
        println!("{}", &args);
        let register_airdrop_return_value: u16 = check_get_value(
            operator
                .call(gradual_release_contract.id(), "register_airdrop")
                .args_json(args)
                .gas(NearGas::from_tgas(50))
                .deposit(NearToken::from_yoctonear(1))
                .transact()
                .await?,
        );
        assert_eq!(register_airdrop_return_value, info.airdrop_index);
    }

    let mut claim_amounts = vec![
        "1200230",
        "200.2",
        "300.3325",
        "400.54556",
        "500",
        "600.15",
        "700.0007",
        "800.25",
        "900",
        "1300100",
    ];

    // create a hashmap with the claims to check later when claiming
    type UserAndAirdrop = (String, u16);
    let mut claims_map: HashMap<UserAndAirdrop, u128> = std::collections::HashMap::new();

    // ----------------
    // register claims for our users
    // ----------------
    for info in tokens.iter() {
        println!("{:?}", info);
        // compute the claims array
        let popped_amounts: Vec<String> = users
            .iter()
            .map(|_| claim_amounts.pop().unwrap().to_string())
            .collect();
        // add the claims
        let sum_claims = popped_amounts.iter().fold(0u128, |prev, string_dec| {
            prev + parse_token_amount(string_dec, info.decimals)
        });
        println!("sum_claims: {}", sum_claims);

        let claim_list = users
            .iter()
            .zip(popped_amounts)
            .map(|(acc, string_dec)| (acc.id().to_string(), string_dec))
            .collect::<Vec<(String, String)>>();
        println!("claim list: {:?}", claim_list);

        // register in the hashmap
        for (acc, string_dec) in claim_list.iter() {
            claims_map.insert(
                (acc.clone(), info.airdrop_index.clone()),
                parse_token_amount(string_dec, info.decimals),
            );
        }

        check(
            operator
                .call(gradual_release_contract.id(), "add_claims")
                .args_json(serde_json::json!({
                    "airdrop_index": info.airdrop_index,
                    "total_amount": U128(sum_claims),
                    "data": claim_list,
                }))
                .gas(NearGas::from_tgas(50))
                .deposit(NearToken::from_yoctonear(1))
                .transact()
                .await?,
        );
    }

    // -------------
    // expect error when try to claim, airdrop is not enabled
    // -------------
    for info in tokens.iter().take(1) {
        let claim_tx = users[0]
            .call(gradual_release_contract.id(), "claim")
            .args_json(serde_json::json!({
                "airdrop_index": info.airdrop_index,
            }))
            .gas(NearGas::from_tgas(50));
        expect_error(
            claim_tx,
            &format!("Airdrop {} is not enabled", info.airdrop_index),
        )
        .await;
    }

    // -------------
    // enable the airdrops
    // -------------
    join_all(tokens.iter().map(|info| {
        operator
            .call(gradual_release_contract.id(), "enable_airdrop")
            .args_json(serde_json::json!({
                "airdrop_index": info.airdrop_index,
            }))
            .gas(NearGas::from_tgas(50))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
    })).await;

    // -------------
    // expect error when try to claim, schedule is not started
    // -------------
    for info in tokens.iter().take(1) {
        let claim_tx = users[0]
            .call(gradual_release_contract.id(), "claim")
            .args_json(serde_json::json!({
                "airdrop_index": info.airdrop_index,
            }))
            .gas(NearGas::from_tgas(50));
        expect_error(claim_tx, &"0 available now.".to_string()).await;
    }

    // -------------------
    // change schedule to start 1 sec ago
    // -------------------
    join_all(tokens.iter().map(|info| {
        operator
            .call(gradual_release_contract.id(), "change_schedule")
            .args_json(serde_json::json!({
                "airdrop_index": info.airdrop_index,
                "start_timestamp_ms": U64(current_timestamp_ms - 1000),
                "end_timestamp_ms": U64(current_timestamp_ms),
            }))
            .gas(NearGas::from_tgas(50))
            .transact()
    })).await;

    // -------------------
    // check the airdrops
    // -------------------
    #[allow(dead_code)]
    #[derive(Debug, serde::Deserialize)]
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

    let view_result_details = gradual_release_contract
        .view("get_airdrops_including_not_enabled")
        .await?;
    // show result parsed as utf8
    // let result_str = str::from_utf8(&view_result_details.result).unwrap();
    // println!("result_str {}", result_str);
    let airdrops = view_result_details.json::<Vec<AirdropJSON>>()?;
    println!("{:?}", airdrops);
    for airdrop in airdrops.iter() {
        assert_eq!(airdrop.enabled, true);
        // no claims yet
        assert_eq!(airdrop.total_claimed.0, 0);

        // verify total_in_claims_per_token
        let view_result_details = gradual_release_contract
            .view("get_total_in_claims_per_token")
            .args_json(serde_json::json!({
                "token_contract": airdrop.token_contract,
            }))
            .await?;
        let total_in_claims_per_token = view_result_details.json::<U128>()?;
        println!(
            "{} total_in_claims_per_token {}",
            airdrop.token_contract, total_in_claims_per_token.0
        );
        // because there's one airdrop per token, this 2 should match
        assert_eq!(total_in_claims_per_token.0, airdrop.total_distributed.0);
    }


    // -------------
    // expect error when try to claim, user "zero" is not registered (storage deposit)
    // "The account zero is not registered"
    // -------------
    expect_error(
        users[0]
            .call(gradual_release_contract.id(), "claim")
            .args_json(serde_json::json!({
                "airdrop_index": 0,
            }))
            .gas(NearGas::from_tgas(150)),
        &format!("The account {} is not registered",users[0].id())
            )
        .await;

    // fix the problem, register the user, all contracts
    join_all(
        tokens.iter().map(|info|
            storage_deposit(&users[0], &info.contract_account_id)
        )
    ).await;

    // -------------
    // re-try the claims
    // -------------
    println!("start claiming");
    for info in tokens.iter() {
        let token_contract = &nep141_contracts[info.airdrop_index as usize];
        for user in users.iter() {
            let prev_balance = ft_balance(token_contract, user.id()).await?;
            // println!("prev_balance: {} {}", prev_balance, info.symbol);

            let claim_tx = user
                .call(gradual_release_contract.id(), "claim")
                .args_json(serde_json::json!({
                    "airdrop_index": info.airdrop_index,
                }))
                .gas(NearGas::from_tgas(150));
            exec(claim_tx).await;

            let expected_amount = *claims_map
                .get(&(user.id().to_string(), info.airdrop_index))
                .unwrap();
            assert!(expected_amount > 0);

            let new_balance = ft_balance(token_contract, user.id()).await?;
            // println!("new_balance: {} {}", new_balance, info.symbol);
            assert_eq!(new_balance, prev_balance + expected_amount);
            println!("{} got {} {}", user.id(), expected_amount, info.symbol);
        }
    }
    println!("end claiming");

    let view_result_details = gradual_release_contract
        .view("get_airdrops_including_not_enabled")
        .await?;
    let airdrops = view_result_details.json::<Vec<AirdropJSON>>()?;
    println!("{:?}", airdrops);
    for airdrop in airdrops.iter() {
        assert_eq!(airdrop.enabled, true);
        assert_eq!(airdrop.total_distributed.0, airdrop.total_claimed.0);

        // verify also total_in_claims_per_token
        let view_result_details = gradual_release_contract
            .view("get_total_in_claims_per_token")
            .args_json(serde_json::json!({
                "token_contract": airdrop.token_contract,
            }))
            .await?;
        let total_in_claims_per_token = view_result_details.json::<U128>()?;
        println!(
            "{} total_in_claims_per_token {}",
            airdrop.token_contract, total_in_claims_per_token.0
        );
        // should be no more claims
        assert_eq!(total_in_claims_per_token.0, 0);
    }

    Ok(())
}
