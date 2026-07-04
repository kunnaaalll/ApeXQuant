#[cfg(test)]
mod tests {
    use super::super::DrawdownTracker;
    use rust_decimal_macros::dec;

    #[test]
    fn test_drawdown_tracking() {
        let mut tracker = DrawdownTracker::new();
        assert_eq!(tracker.current_drawdown, dec!(0.0));

        // Peak equity set to 10000
        tracker.observe(dec!(10000.0));
        assert_eq!(tracker.peak_equity, dec!(10000.0));
        assert_eq!(tracker.current_drawdown, dec!(0.0));

        // Drawdown to 9000 (10%)
        tracker.observe(dec!(9000.0));
        assert_eq!(tracker.current_drawdown, dec!(0.10));
        assert_eq!(tracker.max_drawdown, dec!(0.10));

        // Recovery to 11000 (New Peak)
        tracker.observe(dec!(11000.0));
        assert_eq!(tracker.current_drawdown, dec!(0.0));
        assert_eq!(tracker.max_drawdown, dec!(0.10));
    }
}
