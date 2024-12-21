use near_gas::NearGas;
use near_sdk::{json_types::U128, serde_json};
use near_workspaces::{
    types::{KeyType, NearToken, SecretKey},
    Account, AccountId, Contract, DevNetwork, Worker,
};

use crate::{check, exec};

pub fn parse_token_amount(amount_string_dec: &str, token_decimals: u8) -> u128 {
    let dec_point_position = amount_string_dec
        .find('.')
        .unwrap_or(amount_string_dec.len());
    let (amount_no_dec_point, current_decimals) = if dec_point_position == amount_string_dec.len() {
        (amount_string_dec.to_string(), 0 as u32)
    } else {
        let current_decimals = amount_string_dec.len() - dec_point_position - 1;
        assert!(
            current_decimals <= token_decimals as usize,
            "Too many decimals in the string amount"
        );
        let mut amount_no_dec_point = amount_string_dec.to_string();
        amount_no_dec_point.remove(dec_point_position);
        (amount_no_dec_point, current_decimals as u32)
    };
    let amount_u128 = amount_no_dec_point.parse::<u128>().unwrap();
    amount_u128 * 10u128.pow(token_decimals as u32 - current_decimals)
}

pub async fn ft_transfer(
    nep_141_contract: &AccountId,
    source: &Account,
    receiver: &Account,
    amount: u128,
) -> anyhow::Result<()> {
    let res = source
        .call(nep_141_contract, "ft_transfer")
        .args_json(serde_json::json!({
           "receiver_id": receiver.id(),
            "amount": U128(amount)
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        panic!(
            "Transfer {} to {} {} err: {:?}\n",
            source.id(),
            receiver.id(),
            amount,
            res
        );
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn ft_balance(nep_141_contract: &Contract, account_id: &AccountId) -> anyhow::Result<u128> {
    let view_result_details = nep_141_contract
        .view("ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": account_id
        }))
        .await?;
    // result is always a UTF-8 JSON value
    let result: String = view_result_details.json()?;
    // print!("ft_balance_of: {} {}\n", account_id, result);
    // parse the result string into a u128
    let balance = result.parse::<u128>().unwrap();
    Ok(balance)
}

pub(crate) const DEV_ACCOUNT_SEED: &str = "testificate";
const NEP141_TEST_TOKEN_FILEPATH: &str = "res/nep141_test_token.wasm";

pub async fn create_nep141_token(
    worker: &Worker<impl DevNetwork>,
    owner: &Account,
    contract_account_id: &AccountId,
    symbol: &String,
    decimals: u8,
    total_supply: u128,
) -> Contract {
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    let wasm_code = std::fs::read(NEP141_TEST_TOKEN_FILEPATH).unwrap();
    let deploy_result = worker
        .create_tla_and_deploy(contract_account_id.clone(), sk, &wasm_code)
        .await;
    let token_contract = deploy_result.unwrap().unwrap();
    // INIT the contract
    check(
        token_contract
            .call("new_default_meta_2")
            .args_json(serde_json::json!({
                "owner_id": owner.id(),
                "symbol": symbol,
                "decimals": decimals,
                "total_supply": U128(total_supply)
            }))
            .transact()
            .await
            .unwrap(),
    );
    token_contract
}

pub async fn storage_deposit(account: &Account, contract_id: &AccountId) {
    println!("storage_deposit acc:{} token:{}", account.id(), contract_id);
    let call = account
        .call(contract_id, "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": account.id(),
        }))
        .gas(NearGas::from_tgas(50))
        .deposit(NearToken::from_millinear(250));
    let _ = exec(call).await;
}

pub async fn register_storage_cartesian(tokens: &Vec<AccountId>, holders: &Vec<&Account>) {
    let mut tasks = Vec::new();
    for token_contract_id in tokens {
        for user in holders {
            tasks.push(storage_deposit(&user, &token_contract_id));
        }
    }
    futures::future::join_all(tasks).await;
}
