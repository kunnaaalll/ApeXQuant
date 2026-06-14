# APEX V3 Portfolio Heat Engine Implementation

The Portfolio Heat Engine is the central measure of portfolio stress and capital pressure in APEX V3. It drives the overarching capital allocation logic, health monitoring, recommendations, and execution recovery behaviors. 

## Design Philosophy

Think like a Chief Risk Officer, not an allocator. 
Before capital can be sized, the macro stress environment must be fully understood.
The PortfolioHeat score provides a 0-100 gauge of complete portfolio stress.

### Portfolio Heat State

- **Cold (0-20):** Stable, high capacity for new allocations.
- **Normal (20-40):** Standard operational state.
- **Warm (40-60):** Elevated risk, allocations may be scaled back.
- **Hot (60-80):** High stress, restrict new entries, prioritize exits.
- **Critical (80-95):** Severe drawdowns or extreme market volatility. Total allocation freeze.
- **Frozen (95-100):** Circuit breaker territory. Aggressive de-risking mode.

## Factor Contributions

The Heat Score is deterministic, calculated as the sum of weighted contributions:

- **Open Risk**: Stress from total outstanding risk across all positions.
- **Drawdown**: Stress originating from current PnL drawdowns.
- **Concentration**: Overexposure in specific sectors, symbols, or currencies.
- **Margin / Leverage**: Stress from excessive margin utilization.
- **Volatility**: Implied and realized market turbulence.
- **Recovery & Streak**: Elevated heat following consecutive losses or large drawdowns.

## Risk Budget

The `RiskBudget` struct tracks portfolio capacity independently from individual positions:
- `max_portfolio_risk`: The hard ceiling for risk.
- `utilized_risk`: Currently deployed risk.
- `reserved_risk`: Pre-allocated risk that has not yet been deployed.
- `emergency_reserve`: Un-touchable risk allocation required to safely navigate drawdowns.

The system ensures: `Remaining Risk = Total Capacity - Utilized - Reserved - Emergency`.

## Decay Model

Portfolio Heat does not reset instantly. Following large losses or drawdowns, the portfolio enters a "Cooldown Period".
The `HeatDecayModel` dictates that a required number of time-ticks must elapse without further losses before the heat score can begin a linear decay back to `Cold`.
