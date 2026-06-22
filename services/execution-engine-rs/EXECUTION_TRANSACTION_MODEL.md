# Execution Transaction Model

Transactions securely wrap sequence updates and snapshot generations together inside atomic bounds.

Any partial write MUST rollback.
We leverage `sqlx::Transaction` across database connections to seamlessly bridge multiple table inserts into one cohesive commit pipeline. 

If event insertion fails, the subsequent snapshot operation must never execute, leaving the previous stable state isolated and valid.
