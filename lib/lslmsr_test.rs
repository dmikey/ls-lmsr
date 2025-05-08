#[cfg(test)]
mod tests {
    use super::super::lslmsr::*;
    use super::super::types::DECIMALS;

    #[test]
    fn test_calc_b() {
        // Test with various alpha and share values
        assert_eq!(calc_b(100, 0), 0);
        assert_eq!(calc_b(100, 100), 1000);
        assert_eq!(calc_b(200, 100), 2000);
        
        // Test with large values
        let large_alpha = 5000 * DECIMALS;
        let large_shares = 10000 * DECIMALS;
        let result = calc_b(large_alpha, large_shares);
        assert!(result > 0);
    }

    #[test]
    fn test_calc_cost() {
        // Test with equal q values
        let b = 1000.0;
        let cost = calc_cost(100.0, 100.0, b);
        // Expected: b * ln(e^(q_yes/b) + e^(q_no/b))
        let expected = b * libm::log(libm::exp(100.0 / b) + libm::exp(100.0 / b));
        assert!((cost - expected).abs() < 0.001);

        // Test with different q values
        let cost_uneven = calc_cost(200.0, 100.0, b);
        assert!(cost_uneven > cost); // Adding more shares should increase cost
    }

    #[test]
    fn test_calc_price() {
        // Test with equal shares (should give 0.5/0.5)
        let (p_yes, p_no) = calc_price(100.0, 100.0, 1000.0);
        assert!((p_yes - 0.5).abs() < 0.001);
        assert!((p_no - 0.5).abs() < 0.001);
        assert!((p_yes + p_no - 1.0).abs() < 0.001); // Should sum to 1

        // Test with uneven shares
        let (p_yes2, p_no2) = calc_price(200.0, 100.0, 1000.0);
        assert!(p_yes2 > p_no2); // More YES shares should give higher YES price
        assert!((p_yes2 + p_no2 - 1.0).abs() < 0.001); // Should still sum to 1
    }
}
