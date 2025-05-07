use crate::lslmsr::*;
use crate::types::*;

pub struct MarketEngine {
    pub alpha: u128,
    pub q_yes: u128,
    pub q_no: u128,
    pub total_collateral: u128,
}

impl MarketEngine {
    pub fn new(alpha: u128) -> Self {
        Self {
            alpha,
            q_yes: 0,
            q_no: 0,
            total_collateral: 0,
        }
    }

    pub fn buy(&mut self, outcome: Outcome, amount: u128) -> Result<Price, TradeError> {
        let old_cost = calc_cost(
            self.q_yes as f64,
            self.q_no as f64,
            calc_b(self.alpha, self.q_yes + self.q_no) as f64,
        );

        match outcome {
            Outcome::Yes => self.q_yes += amount,
            Outcome::No => self.q_no += amount,
        }

        let new_cost = calc_cost(
            self.q_yes as f64,
            self.q_no as f64,
            calc_b(self.alpha, self.q_yes + self.q_no) as f64,
        );

        let cost = new_cost - old_cost;
        self.total_collateral += cost as u128;

        let (p_yes, p_no) = calc_price(self.q_yes as f64, self.q_no as f64, calc_b(self.alpha, self.q_yes + self.q_no) as f64);

        Ok(Price {
            yes: (p_yes * DECIMALS as f64) as u128,
            no: (p_no * DECIMALS as f64) as u128,
        })
    }

    pub fn get_price(&self) -> Price {
        let (p_yes, p_no) = calc_price(
            self.q_yes as f64,
            self.q_no as f64,
            calc_b(self.alpha, self.q_yes + self.q_no) as f64,
        );

        Price {
            yes: (p_yes * DECIMALS as f64) as u128,
            no: (p_no * DECIMALS as f64) as u128,
        }
    }

    pub fn simulate(&self, outcome: Outcome, amount: u128) -> u128 {
        let mut q_yes = self.q_yes;
        let mut q_no = self.q_no;

        match outcome {
            Outcome::Yes => q_yes += amount,
            Outcome::No => q_no += amount,
        }

        let b_old = calc_b(self.alpha, self.q_yes + self.q_no) as f64;
        let b_new = calc_b(self.alpha, q_yes + q_no) as f64;

        let old_cost = calc_cost(self.q_yes as f64, self.q_no as f64, b_old);
        let new_cost = calc_cost(q_yes as f64, q_no as f64, b_new);

        ((new_cost - old_cost) * DECIMALS as f64) as u128
    }
}