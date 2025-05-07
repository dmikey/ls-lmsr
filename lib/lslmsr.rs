use crate::types::DECIMALS;
use libm::{exp, ln, sqrt}; // or fixed-point wrappers

pub fn calc_b(alpha: u128, total_shares: u128) -> u128 {
    let sqrt_total = sqrt(total_shares as f64);
    (alpha as f64 * sqrt_total * DECIMALS as f64) as u128 / DECIMALS
}

pub fn calc_cost(q_yes: f64, q_no: f64, b: f64) -> f64 {
    b * ( (q_yes / b).exp() + (q_no / b).exp() ).ln()
}

pub fn calc_price(q_yes: f64, q_no: f64, b: f64) -> (f64, f64) {
    let exp_yes = (q_yes / b).exp();
    let exp_no = (q_no / b).exp();
    let denom = exp_yes + exp_no;
    (exp_yes / denom, exp_no / denom)
}