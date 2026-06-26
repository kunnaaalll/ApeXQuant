#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use crate::volatility::VolatilityEngine;
    use crate::trend::TrendEngine;
    use crate::momentum::MomentumEngine;
    use crate::structure::StructureEngine;
    use crate::correlation::CorrelationEngine;
    use crate::regime::RegimeEngine;
    use crate::quality::QualityEngine;
    use crate::intelligence::IntelligenceAggregator;
    use crate::validation::ValidationFramework;

    #[test]
    fn test_determinism_100k_iterations() -> Result<(), &'static str> {
        let mut vol_engine = VolatilityEngine::new(14);
        let mut trend_engine = TrendEngine::new(10, 30);
        let mut mom_engine = MomentumEngine::new(14);
        let mut struct_engine = StructureEngine::new(20);
        let mut corr_engine = CorrelationEngine::new(20);
        let mut regime_engine = RegimeEngine::new();
        let quality_engine = QualityEngine::new();

        let mut final_score = 0;

        for i in 0..100_000 {
            // Deterministic pseudo-random price movement using modulus
            let price = Decimal::from(10000 + (i % 100));
            let high = price + Decimal::from(10);
            let low = price - Decimal::from(10);
            let ret_a = if i % 2 == 0 { Decimal::from(1) } else { Decimal::from(-1) };
            let ret_b = if i % 3 == 0 { Decimal::from(1) } else { Decimal::from(-1) };

            let vol = vol_engine.update(high, low, price)?;
            let trend = trend_engine.update(price)?;
            let mom = mom_engine.update(price)?;
            let structure = struct_engine.update(high, low)?;
            let corr = corr_engine.update(ret_a, ret_b)?;
            let regime = regime_engine.determine_regime(
                vol.is_expanding,
                vol.is_contracting,
                trend.strength_score,
                structure.state,
            )?;
            
            let quality = quality_engine.evaluate(
                Decimal::from(1),
                Decimal::from(1),
                Decimal::from(100),
                Decimal::from(100),
                true,
                0,
                Decimal::from(1),
                Decimal::from(1),
                Decimal::from(1),
                Decimal::from(1),
            )?;

            let profile = IntelligenceAggregator::build_profile(
                vol,
                trend,
                mom,
                structure,
                corr,
                regime,
                quality,
            )?;

            ValidationFramework::validate_bounds(&profile)?;
            assert!(ValidationFramework::validate_regime(profile.regime.current_regime));
            ValidationFramework::validate_correlation(&profile.correlation)?;

            if i == 99_999 {
                final_score = profile.market_score;
            }
        }

        // Assert it always outputs exactly this deterministic final score
        assert_eq!(final_score, 18);
        Ok(())
    }
}
