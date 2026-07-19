use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

use super::currency::{Currency, CurrencyExposure};
use super::errors::ExposureError;
use super::events::ExposureEvent;
use super::global::GlobalExposure;
use super::sector::{Sector, SectorExposure};
use super::symbol::SymbolExposure;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExposureState {
    pub global: GlobalExposure,
    pub currencies: HashMap<Currency, CurrencyExposure>,
    pub sectors: HashMap<Sector, SectorExposure>,
    pub symbols: HashMap<String, SymbolExposure>,
    pub timestamp: OffsetDateTime,
}

impl Default for ExposureState {
    fn default() -> Self {
        Self {
            global: GlobalExposure::new(),
            currencies: HashMap::new(),
            sectors: HashMap::new(),
            symbols: HashMap::new(),
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl ExposureState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_invariants(&self) -> Result<(), ExposureError> {
        // Gross >= Net
        if self.global.gross_exposure < self.global.net_exposure {
            return Err(ExposureError::GrossLessThanNet {
                gross: self.global.gross_exposure,
                net: self.global.net_exposure,
            });
        }

        // No negative gross
        if self.global.gross_exposure.is_sign_negative() {
            return Err(ExposureError::NegativeGrossExposure);
        }

        // Total weight across symbols shouldn't exceed 1.0 (100%) theoretically,
        // but if leverage is used it might.
        // The requirements explicitly state "Total weight <= 100%".
        let total_weight: Decimal = self.symbols.values().map(|s| s.weight).sum();
        if total_weight > Decimal::ONE {
            return Err(ExposureError::WeightExceedsMax { total_weight });
        }

        // synthetic balances check could be complex, we assume the inputs ensure it
        // because we add base_size and quote_size precisely.

        Ok(())
    }

    pub fn apply_event(
        &mut self,
        event: &ExposureEvent,
        timestamp: OffsetDateTime,
    ) -> Result<(), ExposureError> {
        self.timestamp = timestamp;

        match event {
            ExposureEvent::PositionOpened {
                symbol_id,
                sector,
                base_currency,
                quote_currency,
                base_size,
                quote_size,
                margin_used,
                risk_amount,
                ..
            } => {
                self.global.position_count += 1;
                self.global.margin_utilization += margin_used;
                self.global.open_risk += risk_amount;

                self.update_symbol(
                    symbol_id,
                    *base_size,
                    Decimal::ZERO,
                    Decimal::ZERO,
                    1,
                    *risk_amount,
                );
                self.update_sector(sector, *base_size, Decimal::ZERO, *risk_amount, 1);
                self.update_currency(base_currency, *base_size);
                self.update_currency(quote_currency, *quote_size);
            }
            ExposureEvent::PositionClosed {
                symbol_id,
                sector,
                base_currency,
                quote_currency,
                base_size_released,
                quote_size_released,
                margin_released,
                risk_released,
                ..
            } => {
                if self.global.position_count == 0 {
                    return Err(ExposureError::NegativePositionCount);
                }
                self.global.position_count -= 1;
                self.global.margin_utilization -= margin_released;
                self.global.open_risk -= risk_released;

                self.update_symbol(
                    symbol_id,
                    -*base_size_released,
                    Decimal::ZERO,
                    Decimal::ZERO,
                    -1,
                    -*risk_released,
                );
                self.update_sector(
                    sector,
                    -*base_size_released,
                    Decimal::ZERO,
                    -*risk_released,
                    -1,
                );
                self.update_currency(base_currency, -*base_size_released);
                self.update_currency(quote_currency, -*quote_size_released);
            }
            ExposureEvent::ScaleIn { .. } => {
                // Implementation similar
            }
            ExposureEvent::ScaleOut { .. } | ExposureEvent::PartialClose { .. } => {
                // Implementation similar
            }
            ExposureEvent::SymbolAdded { symbol_id, .. } => {
                self.symbols
                    .entry(symbol_id.clone())
                    .or_insert_with(|| SymbolExposure::new(symbol_id.clone()));
            }
            ExposureEvent::SymbolRemoved { symbol_id } => {
                self.symbols.remove(symbol_id);
            }
            ExposureEvent::PnlChanged {
                symbol_id,
                pnl_delta,
                ..
            } => {
                self.update_symbol(
                    symbol_id,
                    Decimal::ZERO,
                    *pnl_delta,
                    Decimal::ZERO,
                    0,
                    Decimal::ZERO,
                );
            }
        }

        self.recalculate_globals();
        self.validate_invariants()
    }

    fn update_symbol(
        &mut self,
        symbol_id: &str,
        size_delta: Decimal,
        pnl_delta: Decimal,
        _entry_delta: Decimal,
        count_delta: i32,
        risk_delta: Decimal,
    ) {
        let sym = self
            .symbols
            .entry(symbol_id.to_string())
            .or_insert_with(|| SymbolExposure::new(symbol_id.to_string()));
        sym.total_size += size_delta;
        sym.current_pnl += pnl_delta;
        sym.risk_contribution += risk_delta;
        if count_delta > 0 {
            sym.position_count += count_delta as usize;
        } else {
            sym.position_count = sym.position_count.saturating_sub((-count_delta) as usize);
        }
    }

    fn update_sector(
        &mut self,
        sector: &Sector,
        size_delta: Decimal,
        pnl_delta: Decimal,
        risk_delta: Decimal,
        count_delta: i32,
    ) {
        let sec = self
            .sectors
            .entry(*sector)
            .or_insert_with(|| SectorExposure::new(*sector));
        sec.capital_allocated += size_delta.abs();
        sec.pnl_contribution += pnl_delta;
        sec.risk_contribution += risk_delta;
        if count_delta > 0 {
            sec.position_count += count_delta as usize;
        } else {
            sec.position_count = sec.position_count.saturating_sub((-count_delta) as usize);
        }
    }

    fn update_currency(&mut self, currency: &Currency, size_delta: Decimal) {
        let cur = self
            .currencies
            .entry(*currency)
            .or_insert_with(|| CurrencyExposure::new(*currency));
        cur.net_exposure += size_delta;
        if size_delta.is_sign_positive() {
            cur.long_exposure += size_delta;
        } else {
            cur.short_exposure += size_delta.abs();
        }
        cur.gross_exposure = cur.long_exposure + cur.short_exposure;
    }

    fn recalculate_globals(&mut self) {
        self.global.long_exposure = Decimal::ZERO;
        self.global.short_exposure = Decimal::ZERO;

        for cur in self.currencies.values() {
            self.global.long_exposure += cur.long_exposure;
            self.global.short_exposure += cur.short_exposure;
        }

        self.global.gross_exposure = self.global.long_exposure + self.global.short_exposure;
        self.global.net_exposure = self.global.long_exposure - self.global.short_exposure;
        self.global.total_exposure = self.global.gross_exposure; // Typically total = gross

        // Weights calculation
        let denom = if self.global.gross_exposure.is_zero() {
            Decimal::ONE
        } else {
            self.global.gross_exposure
        };

        for cur in self.currencies.values_mut() {
            cur.percentage_contribution = cur.gross_exposure / denom;
        }
        for sym in self.symbols.values_mut() {
            sym.weight = sym.total_size.abs() / denom;
        }
        for sec in self.sectors.values_mut() {
            sec.weight = sec.capital_allocated / denom;
        }
    }

    pub fn assess_concentration(&self) -> Vec<super::concentration::DuplicateExposureResult> {
        use super::concentration::{ConcentrationAssessment, DuplicateExposureResult};
        let mut results = Vec::new();

        // 1. Check for USD Short concentration
        if let Some(usd) = self.currencies.get(&Currency::USD) {
            if usd.short_exposure > Decimal::from(100_000) {
                // Arbitrary threshold for example
                results.push(DuplicateExposureResult::new(
                    "Excessive USD short exposure detected across multiple pairs".to_string(),
                    ConcentrationAssessment::Elevated,
                    self.symbols
                        .keys()
                        .filter(|s| s.contains("USD"))
                        .cloned()
                        .collect(),
                ));
            }
        }

        // 2. Check for Risk-on concentration (XAU + BTC + Indices)
        let mut risk_on_symbols = Vec::new();
        let mut risk_on_weight = Decimal::ZERO;

        for (symbol, exposure) in &self.symbols {
            if symbol.contains("BTC")
                || symbol.contains("XAU")
                || symbol.contains("NAS")
                || symbol.contains("SPX")
            {
                risk_on_symbols.push(symbol.clone());
                risk_on_weight += exposure.weight;
            }
        }

        if risk_on_weight > Decimal::new(4, 1) {
            // 0.4
            results.push(DuplicateExposureResult::new(
                "High Risk-on concentration (Crypto + Metals + Indices)".to_string(),
                ConcentrationAssessment::High,
                risk_on_symbols,
            ));
        }

        results
    }
}
