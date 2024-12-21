use crate::*;
use uint::construct_uint;

#[inline]
pub fn get_current_epoch_millis() -> u64 {
    env::block_timestamp() / 1_000_000
}

// #[inline]
// pub fn days_to_millis(days: Days) -> u64 {
//     (days as u64) * 24 * 60 * 60 * 1_000
// }

// #[inline]
// pub fn millis_to_days(millis: u64) -> Days {
//     (millis / (24 * 60 * 60 * 1_000)) as Days
// }

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[inline]
/// returns amount * numerator/denominator
pub fn proportional(amount: u128, numerator: u128, denominator: u128) -> u128 {
    (U256::from(amount) * U256::from(numerator) / U256::from(denominator)).as_u128()
}

pub fn parse_token_amount(amount_string: &str, token_decimals: u8) -> u128 {
    let dec_point_position = amount_string.find('.').unwrap_or(amount_string.len());
    let (amount_no_dec_point, current_decimals) = if dec_point_position == amount_string.len() {
        (amount_string.to_string(), 0 as u32)
    } else {
        let current_decimals = amount_string.len() - dec_point_position - 1;
        assert!(
            current_decimals <= token_decimals as usize,
            "Too many decimals in the string amount"
        );
        let mut amount_no_dec_point = amount_string.to_string();
        amount_no_dec_point.remove(dec_point_position);
        (amount_no_dec_point, current_decimals as u32)
    };
    let amount_u128 = amount_no_dec_point.parse::<u128>().unwrap();
    amount_u128 * 10u128.pow(token_decimals as u32 - current_decimals)
}
