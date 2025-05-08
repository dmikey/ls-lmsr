// constants
pub const DECIMALS: u128 = 1_000_000_000_000_000_000; // 1e18

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Yes,
    No,
}

#[derive(Debug)]
pub struct Price {
    pub yes: u128,
    pub no: u128,
}

#[derive(Debug)]
pub struct MarketState {
    pub q_yes: u128,
    pub q_no: u128,
    pub total_collateral: u128,
}

#[derive(Debug)]
pub enum TradeError {
    InvalidOutcome,
    InsufficientCollateral,
}