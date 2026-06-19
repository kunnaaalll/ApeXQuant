# Strategy State Implementation

Defines the strategy state machine, encompassing 10 states: `Dormant`, `Research`, `Emerging`, `Active`, `Strong`, `Elite`, `Weakening`, `Deteriorating`, `Paused`, `Retired`.
Explicitly prevents illegal transitions such as `Retired -> Active`.
