pub mod breakeven;
pub mod partial_close;
pub mod stop_loss;
pub mod take_profit;
pub mod trailing_stop;

// Explainability requirement: Every action must answer "Why?"
pub trait ExplainableAction {
    fn reason(&self) -> String;
}
