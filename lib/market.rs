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
        // Calculate cost before the buy
        let old_cost = if self.q_yes == 0 && self.q_no == 0 {
            0.0 // If no shares, starting cost is 0
        } else {
            calc_cost(
                self.q_yes as f64,
                self.q_no as f64,
                calc_b(self.alpha, self.q_yes + self.q_no) as f64,
            )
        };

        // Update shares based on outcome
        match outcome {
            Outcome::Yes => self.q_yes += amount,
            Outcome::No => self.q_no += amount,
        }

        // Calculate new cost
        let new_cost = calc_cost(
            self.q_yes as f64,
            self.q_no as f64,
            calc_b(self.alpha, self.q_yes + self.q_no) as f64,
        );

        // Calculate cost difference and update total collateral
        let cost_diff = new_cost - old_cost;
        let cost_to_add = (cost_diff * DECIMALS as f64) as u128;
        self.total_collateral += cost_to_add;

        // Get current price after buy
        Ok(self.get_price())
    }

    pub fn get_price(&self) -> Price {
        // Handle the case where there are no shares
        if self.q_yes == 0 && self.q_no == 0 {
            return Price {
                yes: DECIMALS / 2,
                no: DECIMALS / 2,
            };
        }

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

        // Handle the case where there are no initial shares
        let old_cost = if self.q_yes == 0 && self.q_no == 0 {
            0.0 // If no shares, starting cost is 0
        } else {
            let b_old = calc_b(self.alpha, self.q_yes + self.q_no) as f64;
            calc_cost(self.q_yes as f64, self.q_no as f64, b_old)
        };

        // Calculate new cost
        let b_new = calc_b(self.alpha, q_yes + q_no) as f64;
        let new_cost = calc_cost(q_yes as f64, q_no as f64, b_new);

        // Return cost difference in fixed-point notation
        ((new_cost - old_cost) * DECIMALS as f64) as u128
    }

    pub fn simulate_sell(&self, outcome: Outcome, amount: u128) -> Result<u128, TradeError> {
        // Validate the user has enough shares to sell
        match outcome {
            Outcome::Yes if self.q_yes < amount => return Err(TradeError::InsufficientCollateral),
            Outcome::No if self.q_no < amount => return Err(TradeError::InsufficientCollateral),
            _ => {}
        }

        let mut q_yes = self.q_yes;
        let mut q_no = self.q_no;

        // Calculate cost before the sell
        let old_cost = calc_cost(
            q_yes as f64,
            q_no as f64,
            calc_b(self.alpha, q_yes + q_no) as f64,
        );

        // Subtract the shares
        match outcome {
            Outcome::Yes => q_yes -= amount,
            Outcome::No => q_no -= amount,
        }

        // Calculate cost after the sell
        let new_cost = if q_yes == 0 && q_no == 0 {
            0.0
        } else {
            calc_cost(
                q_yes as f64,
                q_no as f64,
                calc_b(self.alpha, q_yes + q_no) as f64,
            )
        };

        // Calculate refund amount
        let refund = (old_cost - new_cost) * DECIMALS as f64;
        Ok(refund as u128)
    }

    pub fn sell(&mut self, outcome: Outcome, amount: u128) -> Result<Price, TradeError> {
        // Validate the user has enough shares to sell
        match outcome {
            Outcome::Yes if self.q_yes < amount => return Err(TradeError::InsufficientCollateral),
            Outcome::No if self.q_no < amount => return Err(TradeError::InsufficientCollateral),
            _ => {}
        }

        // Calculate cost before the sell
        let old_cost = calc_cost(
            self.q_yes as f64,
            self.q_no as f64,
            calc_b(self.alpha, self.q_yes + self.q_no) as f64,
        );

        // Subtract the shares
        match outcome {
            Outcome::Yes => self.q_yes -= amount,
            Outcome::No => self.q_no -= amount,
        }

        // Calculate cost after the sell
        let new_cost = if self.q_yes == 0 && self.q_no == 0 {
            0.0
        } else {
            calc_cost(
                self.q_yes as f64,
                self.q_no as f64,
                calc_b(self.alpha, self.q_yes + self.q_no) as f64,
            )
        };

        // Reduce collateral
        let refund = (old_cost - new_cost) * DECIMALS as f64;
        self.total_collateral = self.total_collateral.saturating_sub(refund as u128);

        // Return the updated price
        Ok(self.get_price())
    }
}