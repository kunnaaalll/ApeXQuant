# Execution Sequence Rules

Monotonic sequence enforcement is vital to execution consistency.

## Acceptable Flow
`1 -> 2 -> 3 -> 4`

## Prohibited Flow (Gaps)
`1 -> 3` throws `StorageError::SequenceViolation`

## Prohibited Flow (Out of Order)
`5 -> 2` throws `StorageError::SequenceViolation`

Sequence metrics must seamlessly bridge in-memory models to PostgreSQL primary/unique keys guaranteeing serialization constraints are unviolated database-side.
