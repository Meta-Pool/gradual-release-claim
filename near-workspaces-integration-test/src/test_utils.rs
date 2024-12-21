use core::panic;

use near_workspaces::{operations::CallTransaction, result::ExecutionFinalResult};

// execute a call transaction and panic if it fails
pub async fn exec(call_tx: CallTransaction) {
    let res = call_tx.transact().await.unwrap();
    check(res);
}

pub fn check(res: ExecutionFinalResult) {
    if res.failures().len() > 0 {
        println!("error: {:#?}", res);
        panic!("execution failed");
    }
}

pub fn check_get_value<T>(res: ExecutionFinalResult) -> T
where
    T: serde::de::DeserializeOwned,
{
    if res.failures().len() > 0 {
        println!("error: {:#?}", res);
        panic!("execution failed");
    }
    let result: T = res.json().unwrap();
    result
}

pub async fn expect_error(call_tx: CallTransaction, err_message: &String) {
    let res = call_tx.transact().await.unwrap();
    if res.failures().len() == 0 {
        panic!("res.failures().len() > 0 -- got 0 failures! {}", err_message);
    }
    for failure in res.failures() {
        let as_result = failure.clone().into_result();
        // println!("failure: {:#?}", as_result);
        let result_as_text = match as_result {
            Ok(_) => panic!("expected error, got success"),
            Err(e) => format!("{:?}",e),
        };
        // println!("result_as_text: {}", result_as_text);
        if result_as_text.contains(err_message) {
            println!("OK: received one failure with expected msg got:{}", result_as_text);
            return;
        }
    };
    println!("Expecting error:{} | got:{:?}", err_message, res.into_result());
    panic!("different error message");
}
