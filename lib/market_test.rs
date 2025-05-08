#[cfg(test)]
mod tests {
    use super::super::market::*;
    use super::super::types::*;
    use super::super::lslmsr::*;

    #[test]
    fn test_market_initialization() {
        let alpha = 100;
        let market = MarketEngine::new(alpha);
        
        assert_eq!(market.alpha, alpha);
        assert_eq!(market.q_yes, 0);
        assert_eq!(market.q_no, 0);
        assert_eq!(market.total_collateral, 0);
    }

    #[test]
    fn test_initial_price() {
        // With zero shares, we need to handle the special case
        // Initial price with 0 shares should be 0.5/0.5
        let market = MarketEngine::new(100);
        
        // Get the actual price
        let price = market.get_price();
        
        // Print debug values to understand what's happening
        println!("Price yes: {}, Price no: {}", price.yes, price.no);
        
        // For initial price, we just check they're equal and sum to DECIMALS
        assert_eq!(price.yes, price.no); // Both should be equal
        assert_eq!(price.yes + price.no, DECIMALS); // Should sum to DECIMALS
        
        // For safety, also check they're close to 50%
        let half_decimals = DECIMALS / 2;
        let tolerance = DECIMALS / 100; // 1% tolerance
        
        assert!((price.yes as i128 - half_decimals as i128).abs() < tolerance as i128);
        assert!((price.no as i128 - half_decimals as i128).abs() < tolerance as i128);
    }

    #[test]
    fn test_buy_shares() {
        let mut market = MarketEngine::new(100);
        let initial_collateral = market.total_collateral;
        
        // First, add some initial liquidity by buying both sides equally
        // This helps avoid the zero-share edge case
        market.buy(Outcome::Yes, 10).unwrap();
        market.buy(Outcome::No, 10).unwrap();
        
        // Reset the initial collateral for our test
        let initial_collateral = market.total_collateral;
        
        // Buy YES shares
        let result = market.buy(Outcome::Yes, 100);
        assert!(result.is_ok());
        let price = result.unwrap();
        
        // Verify state changes
        assert_eq!(market.q_yes, 110); // 10 initial + 100 new
        assert_eq!(market.q_no, 10);
        assert!(market.total_collateral > initial_collateral, 
                "Collateral should increase when buying shares. Was {} now {}", 
                initial_collateral, market.total_collateral);
        
        // Prices should reflect the buy (YES price should be higher)
        assert!(price.yes > price.no);
        
        // Buy NO shares to rebalance
        let result2 = market.buy(Outcome::No, 100);
        assert!(result2.is_ok());
        let price2 = result2.unwrap();
        
        // Prices should be closer now
        let price_diff = (price2.yes as i128 - price2.no as i128).abs();
        let initial_diff = (price.yes as i128 - price.no as i128).abs();
        assert!(price_diff < initial_diff);
    }

    #[test]
    fn test_simulate_buy() {
        // Create market with some initial liquidity to avoid zero-share edge case
        let mut market = MarketEngine::new(100);
        market.buy(Outcome::Yes, 10).unwrap();
        market.buy(Outcome::No, 10).unwrap();
        
        // Store the current state
        let initial_yes = market.q_yes;
        let initial_no = market.q_no;
        let initial_collateral = market.total_collateral;
        
        // Simulate buying YES shares
        let cost = market.simulate(Outcome::Yes, 100);
        assert!(cost > 0, "Simulated cost should be positive, was {}", cost);
        
        // The actual market state should remain unchanged
        assert_eq!(market.q_yes, initial_yes);
        assert_eq!(market.q_no, initial_no);
        assert_eq!(market.total_collateral, initial_collateral);
        
        // Now do the actual buy
        market.buy(Outcome::Yes, 100).unwrap();
        
        // The cost from simulate should be close to the actual cost
        let actual_cost = market.total_collateral - initial_collateral;
        let tolerance = cost / 100; // Allow 1% difference
        
        println!("Simulated cost: {}, Actual cost: {}", cost, actual_cost);
        assert!((cost as i128 - actual_cost as i128).abs() < tolerance as i128, 
                "Simulated cost {} should be close to actual cost {}", 
                cost, actual_cost);
    }

    #[test]
    fn test_edge_cases() {
        // Test with zero shares (edge case)
        let market = MarketEngine::new(100);
        
        // Get price
        let price = market.get_price();
        assert_eq!(price.yes, DECIMALS / 2);
        assert_eq!(price.no, DECIMALS / 2);
        
        // First buy should work properly even with zero initial shares
        let mut market = MarketEngine::new(100);
        let result = market.buy(Outcome::Yes, 100);
        assert!(result.is_ok());
        assert!(market.total_collateral > 0);
        
        // Price after buy should favor YES
        let price_after = market.get_price();
        assert!(price_after.yes > price_after.no);
    }
}
