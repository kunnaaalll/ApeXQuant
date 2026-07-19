use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternState {
    Exceptional,
    Strong,
    Normal,
    Weak,
    Negative,
}

pub struct PatternEvaluateParams {
    pub pattern_name: String,
    pub trade_count: u32,
    pub wins: u32,
    pub _losses: u32,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub max_drawdown: Decimal,
}

#[derive(Debug, Clone)]
pub struct PatternAssessment {
    pub pattern_name: String,
    pub trade_count: u32,
    pub win_rate: Decimal,
    pub average_rr: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown: Decimal,
    pub sample_quality: String,
    pub confidence: Decimal,
    pub state: PatternState,
}

impl PatternAssessment {
    pub fn evaluate(params: PatternEvaluateParams) -> Self {
        let trade_count = params.trade_count;
        let average_win = params.average_win;
        let average_loss = params.average_loss;
        let gross_profit = params.gross_profit;
        let gross_loss = params.gross_loss;
        let max_drawdown = params.max_drawdown;
        let pattern_name = params.pattern_name;

        let win_rate = if trade_count > 0 {
            Decimal::from(params.wins) / Decimal::from(trade_count)
        } else {
            Decimal::ZERO
        };

        let average_rr = if average_loss > Decimal::ZERO {
            average_win / average_loss
        } else if average_win > Decimal::ZERO {
            dec!(99.99) // bounded safe max
        } else {
            Decimal::ZERO
        };

        let loss_rate = dec!(1.0) - win_rate;
        let expectancy = (win_rate * average_win) - (loss_rate * average_loss);

        let profit_factor = if gross_loss > Decimal::ZERO {
            gross_profit / gross_loss
        } else if gross_profit > Decimal::ZERO {
            dec!(99.99)
        } else {
            Decimal::ZERO
        };

        let state = Self::derive_state(expectancy, profit_factor, win_rate);

        // Derive raw confidence based on state
        let raw_confidence = match state {
            PatternState::Exceptional => dec!(1.0),
            PatternState::Strong => dec!(0.8),
            PatternState::Normal => dec!(0.6),
            PatternState::Weak => dec!(0.3),
            PatternState::Negative => dec!(0.1),
        };

        let (sample_quality, confidence) = Self::apply_sample_penalty(raw_confidence, trade_count);

        Self {
            pattern_name,
            trade_count,
            win_rate,
            average_rr,
            expectancy,
            profit_factor,
            max_drawdown,
            sample_quality,
            confidence,
            state,
        }
    }

    fn derive_state(
        expectancy: Decimal,
        profit_factor: Decimal,
        win_rate: Decimal,
    ) -> PatternState {
        if expectancy < Decimal::ZERO || profit_factor < dec!(1.0) {
            PatternState::Negative
        } else if expectancy > dec!(0.5) && profit_factor > dec!(2.0) && win_rate > dec!(0.4) {
            PatternState::Exceptional
        } else if expectancy > dec!(0.2) && profit_factor > dec!(1.5) {
            PatternState::Strong
        } else if expectancy > dec!(0.05) && profit_factor > dec!(1.1) {
            PatternState::Normal
        } else {
            PatternState::Weak
        }
    }

    fn apply_sample_penalty(raw_confidence: Decimal, trade_count: u32) -> (String, Decimal) {
        if trade_count < 20 {
            ("Insufficient".to_string(), raw_confidence * dec!(0.1))
        } else if trade_count < 50 {
            ("Weak".to_string(), raw_confidence * dec!(0.5))
        } else if trade_count < 100 {
            ("Acceptable".to_string(), raw_confidence * dec!(0.8))
        } else if trade_count < 300 {
            ("Strong".to_string(), raw_confidence)
        } else {
            ("Institutional".to_string(), raw_confidence)
        }
    }
}
