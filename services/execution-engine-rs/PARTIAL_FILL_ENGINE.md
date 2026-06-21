# Partial Fill Engine

Maintains states (`None`, `Partial`, `Completed`) based on exact quantity accumulated.
Rejects any fill that causes `filled_quantity > requested_quantity`.

## Average Fill Price
Calculates a volume-weighted average fill price cumulatively as each fill arrives.
Uses `rust_decimal::Decimal` and truncates to 8 decimal places max for exact matching.
