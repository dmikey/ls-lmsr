use crate::types::DECIMALS;
use libm::{log, sqrt}; // Removed unused 'exp' import

pub fn calc_b(alpha: u128, total_shares: u128) -> u128 {
    let sqrt_total = sqrt(total_shares as f64);
    (alpha as f64 * sqrt_total * DECIMALS as f64) as u128 / DECIMALS
}

pub fn calc_cost(q_yes: f64, q_no: f64, b: f64) -> f64 {
    b * log(libm::exp(q_yes / b) + libm::exp(q_no / b)) // Use libm::exp directly
}

pub fn calc_price(q_yes: f64, q_no: f64, b: f64) -> (f64, f64) {
    let exp_yes = libm::exp(q_yes / b); // Use libm::exp directly
    let exp_no = libm::exp(q_no / b);   // Use libm::exp directly
    let denom = exp_yes + exp_no;
    (exp_yes / denom, exp_no / denom)
}