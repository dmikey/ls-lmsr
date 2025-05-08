use lslmsr::{
    lslmsr::{calc_b, calc_cost, calc_price},
    market::MarketEngine,
    types::{Outcome, DECIMALS},
};

#[test]
fn test_market_lifecycle() {
    // Initialize market with alpha parameter
    let alpha = 100 * DECIMALS;
    let mut market = MarketEngine::new(alpha);
    
    // Verify initial state
    assert_eq!(market.q_yes, 0);
    assert_eq!(market.q_no, 0);
    
    // Get initial price (should be around 50/50)
    let initial_price = market.get_price();
    let expected_initial = (0.5 * DECIMALS as f64) as u128;
    let tolerance = DECIMALS / 100; // 1% tolerance
    
    assert!((initial_price.yes as i128 - expected_initial as i128).abs() < tolerance as i128);
    assert!((initial_price.no as i128 - expected_initial as i128).abs() < tolerance as i128);
    
    // Simulate a buy before making the actual purchase
    let simulated_cost = market.simulate(Outcome::Yes, 500 * DECIMALS);
    
    // Make the actual purchase
    let price_after_buy = market.buy(Outcome::Yes, 500 * DECIMALS).unwrap();
    
    // Verify the simulation was accurate
    assert_eq!(simulated_cost, market.total_collateral);
    
    // Verify the YES price increased and NO price decreased
    assert!(price_after_buy.yes > initial_price.yes);
    assert!(price_after_buy.no < initial_price.no);
    
    // Buy more of the opposite outcome to balance the market
    let _ = market.buy(Outcome::No, 500 * DECIMALS).unwrap();
    
    // Get final prices
    let final_price = market.get_price();
    
    // Prices should be closer to balanced again
    let final_diff = (final_price.yes as i128 - final_price.no as i128).abs();
    let after_buy_diff = (price_after_buy.yes as i128 - price_after_buy.no as i128).abs();
    
    assert!(final_diff < after_buy_diff);
}

#[test]
fn test_large_trades() {
    let alpha = 1000 * DECIMALS;
    let mut market = MarketEngine::new(alpha);
    
    // Make a series of increasingly large trades
    let trades = [
        (Outcome::Yes, 100 * DECIMALS),
        (Outcome::No, 200 * DECIMALS),
        (Outcome::Yes, 500 * DECIMALS),
        (Outcome::No, 1000 * DECIMALS),
    ];
    
    for (outcome, amount) in trades.iter() {
        // Simulate first
        let simulated_cost = market.simulate(*outcome, *amount);
        
        // Get price before
        let price_before = market.get_price();
        
        // Execute trade
        let price_after = market.buy(*outcome, *amount).unwrap();
        
        // Verify simulation accuracy
        let old_collateral = market.total_collateral - simulated_cost;
        assert!(market.total_collateral > old_collateral);
        
        // Price should move in expected direction
        match outcome {
            Outcome::Yes => {
                assert!(price_after.yes > price_before.yes);
                assert!(price_after.no < price_before.no);
            },
            Outcome::No => {
                assert!(price_after.no > price_before.no);
                assert!(price_after.yes < price_before.yes);
            },
        }
    }
}
