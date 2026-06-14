//! Sector exposure analysis
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Currency sectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sector {
    /// Major currencies (USD, EUR, JPY, GBP)
    Major,
    /// Commodity currencies (AUD, NZD, CAD)
    Commodity,
    /// Safe haven currencies (CHF, JPY)
    SafeHaven,
    /// European currencies (EUR, GBP, CHF, NOK, SEK)
    European,
    /// Asia-Pacific (JPY, AUD, NZD, SGD)
    AsiaPacific,
    /// Emerging markets
    Emerging,
    /// Cross pairs (no USD)
    Cross,
}

impl Sector {
    /// Classify a currency into sectors
    pub fn for_currency(currency: &str) -> Vec<Self> {
        let mut sectors = Vec::new();

        match currency.to_uppercase().as_str() {
            "USD" => sectors.push(Sector::Major),
            "EUR" => {
                sectors.push(Sector::Major);
                sectors.push(Sector::European);
            }
            "GBP" => {
                sectors.push(Sector::Major);
                sectors.push(Sector::European);
            }
            "JPY" => {
                sectors.push(Sector::Major);
                sectors.push(Sector::SafeHaven);
                sectors.push(Sector::AsiaPacific);
            }
            "CHF" => {
                sectors.push(Sector::SafeHaven);
                sectors.push(Sector::European);
            }
            "AUD" | "NZD" => {
                sectors.push(Sector::Commodity);
                sectors.push(Sector::AsiaPacific);
            }
            "CAD" => {
                sectors.push(Sector::Commodity);
            }
            "NOK" | "SEK" => {
                sectors.push(Sector::European);
            }
            "SGD" | "CNY" | "HKD" => {
                sectors.push(Sector::AsiaPacific);
            }
            _ => sectors.push(Sector::Emerging),
        }

        sectors
    }

    /// Classify a full symbol
    pub fn for_symbol(symbol: &str) -> Vec<Self> {
        let mut sectors = HashMap::new();

        if symbol.len() >= 6 {
            let base = &symbol[0..3];
            let quote = &symbol[3..6];

            // Check if cross pair
            if base != "USD" && quote != "USD" {
                sectors.insert(Sector::Cross, ());
            }

            // Add sectors for both currencies
            for curr_sector in Self::for_currency(base) {
                sectors.insert(curr_sector, ());
            }
            for curr_sector in Self::for_currency(quote) {
                sectors.insert(curr_sector, ());
            }
        }

        sectors.into_keys().collect()
    }
}

/// Sector exposure analyzer
pub struct SectorExposureAnalyzer;

/// Sector exposure result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorExposure {
    /// Exposure by sector
    pub breakdown: HashMap<Sector, Decimal>,
    /// Primary sector
    pub primary_sector: Option<Sector>,
    /// Sector concentration risk
    pub concentration_risk: Decimal,
}

impl SectorExposureAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze sector exposure
    pub fn analyze(&self, new_symbol: &str, positions: &[(String, Decimal, i8)]) -> SectorExposure {
        let mut breakdown: HashMap<Sector, Decimal> = HashMap::new();

        // Add existing positions
        for (symbol, size, _direction) in positions {
            let sectors = Sector::for_symbol(symbol);
            for sector in sectors {
                *breakdown.entry(sector).or_insert(Decimal::ZERO) += size.abs();
            }
        }

        // Add proposed position
        let new_sectors = Sector::for_symbol(new_symbol);
        for sector in new_sectors {
            *breakdown.entry(sector).or_insert(Decimal::ZERO) += Decimal::ONE;
        }

        // Find primary sector
        let primary_sector = breakdown
            .iter()
            .max_by_key(|(_, v)| v.abs())
            .map(|(k, _)| *k);

        // Calculate concentration risk
        let total: Decimal = breakdown.values().sum();
        let concentration_risk = if total > Decimal::ZERO {
            let max = breakdown.values().copied().max().unwrap_or(Decimal::ZERO);
            max / total
        } else {
            Decimal::ZERO
        };

        SectorExposure {
            breakdown,
            primary_sector,
            concentration_risk,
        }
    }

    /// Check if exposure is concentrated in a single sector
    pub fn is_concentrated(&self, exposure: &SectorExposure, threshold: Decimal) -> bool {
        exposure.concentration_risk > threshold
    }

    /// Get exposure for a specific sector
    pub fn get_sector_exposure(&self, exposure: &SectorExposure, sector: Sector) -> Decimal {
        exposure.breakdown.get(&sector).copied().unwrap_or(Decimal::ZERO)
    }

    /// Calculate sector diversification score (0-1, higher = more diversified)
    pub fn diversification_score(&self, exposure: &SectorExposure) -> Decimal {
        let total: Decimal = exposure.breakdown.values().sum();

        if total == Decimal::ZERO {
            return Decimal::ZERO;
        }

        // HHI-style concentration index
        let mut hhi = Decimal::ZERO;
        for (_, exposure) in &exposure.breakdown {
            let share = *exposure / total;
            hhi += share * share;
        }

        // Convert to diversification (inverse of concentration)
        Decimal::ONE - hhi
    }

    /// Check for sector-specific risks
    pub fn sector_warnings(&self, exposure: &SectorExposure) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check commodity concentration
        if let Some(&commodity_exposure) = exposure.breakdown.get(&Sector::Commodity) {
            let total: Decimal = exposure.breakdown.values().sum();
            let commodity_pct = commodity_exposure / total;

            if commodity_pct > Decimal::from_f64(0.5).unwrap() {
                warnings.push("High commodity currency exposure".to_string());
            }
        }

        // Check safe haven concentration
        if let Some(&safe_exposure) = exposure.breakdown.get(&Sector::SafeHaven) {
            let total: Decimal = exposure.breakdown.values().sum();
            let safe_pct = safe_exposure / total;

            if safe_pct > Decimal::from_f64(0.6).unwrap() {
                warnings.push("Safe haven concentration - risk-off positioning".to_string());
            }
        }

        // Check cross-pair concentration (lower liquidity)
        if let Some(&cross_exposure) = exposure.breakdown.get(&Sector::Cross) {
            if cross_exposure > Decimal::from(2) {
                warnings.push("Multiple cross pairs - lower liquidity".to_string());
            }
        }

        warnings
    }
}

impl Default for SectorExposureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_sectors() {
        let aud_sectors = Sector::for_currency("AUD");
        assert!(aud_sectors.contains(&Sector::Commodity));
        assert!(aud_sectors.contains(&Sector::AsiaPacific));
    }

    #[test]
    fn test_symbol_sectors() {
        let eurusd_sectors = Sector::for_symbol("EURUSD");
        assert!(eurusd_sectors.contains(&Sector::Major));
        assert!(eurusd_sectors.contains(&Sector::European));
    }

    #[test]
    fn test_cross_pair_detection() {
        let cross_sectors = Sector::for_symbol("EURGBP");
        assert!(cross_sectors.contains(&Sector::Cross));
    }

    #[test]
    fn test_sector_exposure() {
        let analyzer = SectorExposureAnalyzer::new();

        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1),
            ("GBPUSD".to_string(), Decimal::from(1), 1),
        ];

        let exposure = analyzer.analyze("AUDUSD", &positions);

        assert!(exposure.breakdown.contains_key(&Sector::Major));
        assert!(exposure.breakdown.contains_key(&Sector::Commodity));
    }

    #[test]
    fn test_diversification_score() {
        let analyzer = SectorExposureAnalyzer::new();

        let positions = vec![
            ("EURUSD".to_string(), Decimal::from(1), 1), // Major, European
            ("USDJPY".to_string(), Decimal::from(1), 1), // Major, Asia, Safe Haven
            ("AUDUSD".to_string(), Decimal::from(1), 1), // Commodity, Asia
        ];

        let exposure = analyzer.analyze("GBPUSD", &positions);
        let score = analyzer.diversification_score(&exposure);

        assert!(score > Decimal::ZERO);
        assert!(score <= Decimal::ONE);
    }
}
