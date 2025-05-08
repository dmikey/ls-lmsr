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

    #[test]
    fn test_sell_shares() {
        // Create market with some initial liquidity
        let mut market = MarketEngine::new(100);
        
        // First, buy some shares so we have something to sell
        // Buy substantially more YES shares to ensure the price differential
        market.buy(Outcome::Yes, 200).unwrap();
        market.buy(Outcome::No, 50).unwrap();
        
        // Record state before selling
        let initial_yes = market.q_yes;
        let initial_no = market.q_no;
        let initial_collateral = market.total_collateral;
        let initial_price = market.get_price();
        
        // Verify initial price (YES should be higher since we bought more YES)
        assert!(initial_price.yes > initial_price.no);
        
        // Sell YES shares
        let result = market.sell(Outcome::Yes, 100);
        assert!(result.is_ok());
        let price = result.unwrap();
        
        // Verify state changes
        assert_eq!(market.q_yes, initial_yes - 100);
        assert_eq!(market.q_no, initial_no); // NO shares shouldn't change
        assert!(market.total_collateral < initial_collateral, 
                "Collateral should decrease when selling shares. Was {} now {}", 
                initial_collateral, market.total_collateral);
        
        // After selling YES shares, YES price should go down
        assert!(price.yes < initial_price.yes,
                "After selling YES shares, YES price should be lower: before={}, after={}",
                initial_price.yes, price.yes);
    }
    
    #[test]
    fn test_sell_insufficient_shares() {
        // Create market with some initial liquidity
        let mut market = MarketEngine::new(100);
        
        // Buy fewer shares than we'll try to sell
        market.buy(Outcome::Yes, 20).unwrap();
        
        // Try to sell more YES shares than we have
        let result = market.sell(Outcome::Yes, 30);
        assert!(result.is_err());
        
        // Verify we get the correct error
        match result {
            Err(TradeError::InsufficientCollateral) => {}, // This is expected
            _ => panic!("Expected InsufficientCollateral error")
        }
        
        // State should remain unchanged
        assert_eq!(market.q_yes, 20);
    }
    
    #[test]
    fn test_simulate_and_sell() {
        // Create market with some initial liquidity
        let mut market = MarketEngine::new(100);
        market.buy(Outcome::Yes, 100).unwrap();
        market.buy(Outcome::No, 100).unwrap();
        
        // Store the current state
        let initial_collateral = market.total_collateral;
        
        // First simulate selling using the simulate_sell function
        let simulated_refund = market.simulate_sell(Outcome::Yes, 50).unwrap();
        
        // Actual sell
        let before_sell_collateral = market.total_collateral;
        market.sell(Outcome::Yes, 50).unwrap();
        let actual_refund = before_sell_collateral - market.total_collateral;
        
        // Verify refund calculation
        let tolerance = simulated_refund / 100; // 1% tolerance
        println!("Simulated refund: {}, Actual refund: {}", simulated_refund, actual_refund);
        assert!((simulated_refund as i128 - actual_refund as i128).abs() < tolerance as i128, 
                "Simulated refund {} should be close to actual refund {}", 
                simulated_refund, actual_refund);
        
        // Buy some YES shares to get back to a balanced state
        market.buy(Outcome::Yes, 50).unwrap();
        
        // We should end up with approximately the initial collateral
        // (slight difference due to price impact)
        let final_collateral = market.total_collateral;
        let diff = (final_collateral as i128 - initial_collateral as i128).abs();
        assert!(diff < (initial_collateral / 10) as i128, 
                "Final collateral should be close to initial: {} vs {}", 
                final_collateral, initial_collateral);
    }
    
    #[test]
    fn test_sell_all_shares() {
        // Test the edge case of selling all shares
        let mut market = MarketEngine::new(100);
        
        // Buy equal amounts of YES and NO
        market.buy(Outcome::Yes, 30).unwrap();
        market.buy(Outcome::No, 30).unwrap();
        
        // Record state
        let initial_collateral = market.total_collateral;
        
        // Sell all YES shares
        market.sell(Outcome::Yes, 30).unwrap();
        
        // Sell all NO shares
        market.sell(Outcome::No, 30).unwrap();
        
        // We should have no shares and almost no collateral (may be tiny rounding errors)
        assert_eq!(market.q_yes, 0);
        assert_eq!(market.q_no, 0);
        
        // Collateral should be very close to zero
        // (allowing for small rounding errors in float calculations)
        assert!(market.total_collateral < 10, 
                "Collateral should be close to 0 after selling all shares, was {}", 
                market.total_collateral);
        
        // Check prices - should be back to 50/50
        let price = market.get_price();
        let half_decimals = DECIMALS / 2;
        let tolerance = DECIMALS / 100; // 1% tolerance
        
        assert!((price.yes as i128 - half_decimals as i128).abs() < tolerance as i128);
        assert!((price.no as i128 - half_decimals as i128).abs() < tolerance as i128);
    }
}
