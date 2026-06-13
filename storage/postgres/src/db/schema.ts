import {
  pgTable,
  serial,
  varchar,
  timestamp,
  integer,
  real,
  boolean,
  jsonb,
  uuid,
  text,
  index,
  uniqueIndex,
  primaryKey,
  decimal,
} from 'drizzle-orm/pg-core';

// ==================== CORE IDENTIFIERS ====================
// All tables use UUID primary keys for global uniqueness

// ==================== MARKET DATA ====================

export const marketData = pgTable(
  'market_data',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    symbol: varchar('symbol', { length: 12 }).notNull(),
    timeframe: varchar('timeframe', { length: 10 }).notNull(),
    timestamp: timestamp('timestamp', { withTimezone: true }).notNull(),
    open: decimal('open', { precision: 20, scale: 8 }).notNull(),
    high: decimal('high', { precision: 20, scale: 8 }).notNull(),
    low: decimal('low', { precision: 20, scale: 8 }).notNull(),
    close: decimal('close', { precision: 20, scale: 8 }).notNull(),
    volume: decimal('volume', { precision: 20, scale: 4 }).notNull(),
    tickCount: integer('tick_count'),
    source: varchar('source', { length: 50 }).notNull(),
    metadata: jsonb('metadata'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    symbolTimeIdx: index('market_data_symbol_time_idx').on(table.symbol, table.timestamp),
    symbolTfIdx: index('market_data_symbol_tf_idx').on(table.symbol, table.timeframe, table.timestamp),
    uniqueCandle: uniqueIndex('market_data_unique_candle_idx').on(
      table.symbol,
      table.timeframe,
      table.timestamp
    ),
  })
);

// ==================== TRADING SESSIONS ====================

export const sessions = pgTable(
  'sessions',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    accountId: varchar('account_id', { length: 100 }).notNull(),
    mode: varchar('mode', { length: 20 }).notNull().$type<'backtest' | 'paper' | 'live' | 'shadow'>(),
    startedAt: timestamp('started_at', { withTimezone: true }).defaultNow().notNull(),
    endedAt: timestamp('ended_at', { withTimezone: true }),
    initialBalance: decimal('initial_balance', { precision: 20, scale: 2 }),
    finalBalance: decimal('final_balance', { precision: 20, scale: 2 }),
    totalTrades: integer('total_trades').default(0),
    winningTrades: integer('winning_trades').default(0),
    grossProfit: decimal('gross_profit', { precision: 20, scale: 2 }),
    grossLoss: decimal('gross_loss', { precision: 20, scale: 2 }),
    maxDrawdown: decimal('max_drawdown', { precision: 10, scale: 4 }),
    status: varchar('status', { length: 20 }).notNull().default('active'),
    metadata: jsonb('metadata'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
    updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    accountIdx: index('sessions_account_idx').on(table.accountId),
    modeIdx: index('sessions_mode_idx').on(table.mode),
    activeIdx: index('sessions_active_idx').on(table.accountId, table.status),
  })
);

// ==================== SIGNALS ====================

export const signals = pgTable(
  'signals',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    signalId: varchar('signal_id', { length: 100 }).notNull().unique(),
    sessionId: uuid('session_id').references(() => sessions.id),
    symbol: varchar('symbol', { length: 12 }).notNull(),
    timeframe: varchar('timeframe', { length: 10 }).notNull(),
    strategyId: varchar('strategy_id', { length: 100 }).notNull(),
    detectedAt: timestamp('detected_at', { withTimezone: true }).notNull(),
    validUntil: timestamp('valid_until', { withTimezone: true }),
    direction: varchar('direction', { length: 4 }).notNull().$type<'buy' | 'sell'>(),
    signalType: varchar('signal_type', { length: 50 }).notNull(),
    entryZoneLow: decimal('entry_zone_low', { precision: 20, scale: 8 }),
    entryZoneHigh: decimal('entry_zone_high', { precision: 20, scale: 8 }),
    suggestedEntry: decimal('suggested_entry', { precision: 20, scale: 8 }),
    stopLoss: decimal('stop_loss', { precision: 20, scale: 8 }),
    takeProfit: decimal('take_profit', { precision: 20, scale: 8 }),
    confluenceScore: decimal('confluence_score', { precision: 3, scale: 1 }),
    confluenceFactors: jsonb('confluence_factors'),
    mtfAlignment: jsonb('mtf_alignment'),
    marketRegime: varchar('market_regime', { length: 50 }),
    regimeConfidence: decimal('regime_confidence', { precision: 4, scale: 3 }),
    validationStatus: varchar('validation_status', { length: 20 }).default('pending'),
    validationScore: decimal('validation_score', { precision: 4, scale: 3 }),
    aiReasoning: text('ai_reasoning'),
    executed: boolean('executed').default(false),
    executionId: uuid('execution_id'),
    rawData: jsonb('raw_data'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    sessionIdx: index('signals_session_idx').on(table.sessionId),
    symbolIdx: index('signals_symbol_idx').on(table.symbol),
    timeIdx: index('signals_time_idx').on(table.detectedAt),
    statusIdx: index('signals_status_idx').on(table.validationStatus),
    executedIdx: index('signals_executed_idx').on(table.executed),
  })
);

// ==================== DECISIONS ====================

export const decisions = pgTable(
  'decisions',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    decisionId: varchar('decision_id', { length: 100 }).notNull().unique(),
    sessionId: uuid('session_id').references(() => sessions.id),
    signalId: uuid('signal_id').references(() => signals.id),
    madeAt: timestamp('made_at', { withTimezone: true }).defaultNow().notNull(),
    decisionType: varchar('decision_type', { length: 50 }).notNull(),
    outcome: varchar('outcome', { length: 20 }).notNull(), // 'proceed', 'reject', 'defer'
    confidence: decimal('confidence', { precision: 4, scale: 3 }),
    reasoning: jsonb('reasoning'),
    factors: jsonb('factors'),
    aiProvidersUsed: jsonb('ai_providers_used'),
    latencyMs: integer('latency_ms'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    sessionIdx: index('decisions_session_idx').on(table.sessionId),
    signalIdx: index('decisions_signal_idx').on(table.signalId),
    outcomeIdx: index('decisions_outcome_idx').on(table.outcome),
  })
);

// ==================== ORDERS ====================

export const orders = pgTable(
  'orders',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    orderId: varchar('order_id', { length: 100 }).notNull().unique(),
    sessionId: uuid('session_id').references(() => sessions.id),
    signalId: uuid('signal_id').references(() => signals.id),
    positionId: uuid('position_id'),
    symbol: varchar('symbol', { length: 12 }).notNull(),
    orderType: varchar('order_type', { length: 20 }).notNull(),
    side: varchar('side', { length: 4 }).notNull().$type<'buy' | 'sell'>(),
    volume: decimal('volume', { precision: 20, scale: 8 }).notNull(),
    requestedPrice: decimal('requested_price', { precision: 20, scale: 8 }),
    stopLoss: decimal('stop_loss', { precision: 20, scale: 8 }),
    takeProfit: decimal('take_profit', { precision: 20, scale: 8 }),
    state: varchar('state', { length: 20 }).notNull().default('draft'),
    brokerOrderId: varchar('broker_order_id', { length: 100 }),
    submittedAt: timestamp('submitted_at', { withTimezone: true }),
    filledAt: timestamp('filled_at', { withTimezone: true }),
    cancelledAt: timestamp('cancelled_at', { withTimezone: true }),
    filledVolume: decimal('filled_volume', { precision: 20, scale: 8 }),
    averageFillPrice: decimal('average_fill_price', { precision: 20, scale: 8 }),
    commission: decimal('commission', { precision: 20, scale: 4 }),
    slippage: decimal('slippage', { precision: 10, scale: 4 }),
    rejectionReason: text('rejection_reason'),
    metadata: jsonb('metadata'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
    updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    sessionIdx: index('orders_session_idx').on(table.sessionId),
    signalIdx: index('orders_signal_idx').on(table.signalId),
    stateIdx: index('orders_state_idx').on(table.state),
    symbolIdx: index('orders_symbol_idx').on(table.symbol),
  })
);

// ==================== POSITIONS ====================

export const positions = pgTable(
  'positions',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    positionId: varchar('position_id', { length: 100 }).notNull().unique(),
    sessionId: uuid('session_id').references(() => sessions.id),
    parentPositionId: uuid('parent_position_id'),
    signalId: uuid('signal_id').references(() => signals.id),
    symbol: varchar('symbol', { length: 12 }).notNull(),
    side: varchar('side', { length: 4 }).notNull().$type<'buy' | 'sell'>(),
    timeframe: varchar('timeframe', { length: 10 }),
    strategyId: varchar('strategy_id', { length: 100 }),
    state: varchar('state', { length: 20 }).notNull().default('opening'),
    initialVolume: decimal('initial_volume', { precision: 20, scale: 8 }).notNull(),
    currentVolume: decimal('current_volume', { precision: 20, scale: 8 }).notNull(),
    closedVolume: decimal('closed_volume', { precision: 20, scale: 8 }).default('0'),
    entryPrice: decimal('entry_price', { precision: 20, scale: 8 }).notNull(),
    currentPrice: decimal('current_price', { precision: 20, scale: 8 }),
    exitPrice: decimal('exit_price', { precision: 20, scale: 8 }),
    stopLoss: decimal('stop_loss', { precision: 20, scale: 8 }),
    takeProfit: decimal('take_profit', { precision: 20, scale: 8 }),
    breakevenPrice: decimal('breakeven_price', { precision: 20, scale: 8 }),
    trailingStopPrice: decimal('trailing_stop_price', { precision: 20, scale: 8 }),
    isBreakeven: boolean('is_breakeven').default(false),
    hasTrailingStop: boolean('has_trailing_stop').default(false),
    unrealizedPnl: decimal('unrealized_pnl', { precision: 20, scale: 4 }),
    realizedPnl: decimal('realized_pnl', { precision: 20, scale: 4 }),
    commissionPaid: decimal('commission_paid', { precision: 20, scale: 4 }),
    swapPaid: decimal('swap_paid', { precision: 20, scale: 4 }),
    grossPnl: decimal('gross_pnl', { precision: 20, scale: 4 }),
    netPnl: decimal('net_pnl', { precision: 20, scale: 4 }),
    returnPercent: decimal('return_percent', { precision: 10, scale: 4 }),
    closeReason: varchar('close_reason', { length: 50 }),
    openedAt: timestamp('opened_at', { withTimezone: true }).notNull(),
    closedAt: timestamp('closed_at', { withTimezone: true }),
    holdingDurationSeconds: integer('holding_duration_seconds'),
    managementData: jsonb('management_data'),
    metadata: jsonb('metadata'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
    updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    sessionIdx: index('positions_session_idx').on(table.sessionId),
    signalIdx: index('positions_signal_idx').on(table.signalId),
    stateIdx: index('positions_state_idx').on(table.state),
    symbolIdx: index('positions_symbol_idx').on(table.symbol),
    openIdx: index('positions_open_idx').on(table.sessionId, table.state),
    closedAtIdx: index('positions_closed_at_idx').on(table.closedAt),
  })
);

// ==================== TRADES (fills) ====================

export const trades = pgTable(
  'trades',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    tradeId: varchar('trade_id', { length: 100 }).notNull().unique(),
    sessionId: uuid('session_id').references(() => sessions.id),
    orderId: uuid('order_id').references(() => orders.id),
    positionId: uuid('position_id').references(() => positions.id),
    symbol: varchar('symbol', { length: 12 }).notNull(),
    side: varchar('side', { length: 4 }).notNull().$type<'buy' | 'sell'>(),
    volume: decimal('volume', { precision: 20, scale: 8 }).notNull(),
    price: decimal('price', { precision: 20, scale: 8 }).notNull(),
    commission: decimal('commission', { precision: 20, scale: 4 }),
    slippage: decimal('slippage', { precision: 10, scale: 4 }),
    brokerExecutionId: varchar('broker_execution_id', { length: 100 }),
    executedAt: timestamp('executed_at', { withTimezone: true }).notNull(),
    metadata: jsonb('metadata'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    sessionIdx: index('trades_session_idx').on(table.sessionId),
    orderIdx: index('trades_order_idx').on(table.orderId),
    positionIdx: index('trades_position_idx').on(table.positionId),
    executedAtIdx: index('trades_executed_at_idx').on(table.executedAt),
  })
);

// ==================== PATTERNS ====================

export const patterns = pgTable(
  'patterns',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    patternId: varchar('pattern_id', { length: 100 }).notNull().unique(),
    signalId: uuid('signal_id').references(() => signals.id),
    symbol: varchar('symbol', { length: 12 }).notNull(),
    timeframe: varchar('timeframe', { length: 10 }).notNull(),
    patternType: varchar('pattern_type', { length: 50 }).notNull(),
    patternSubtype: varchar('pattern_subtype', { length: 50 }),
    detectedAt: timestamp('detected_at', { withTimezone: true }).notNull(),
    startBar: timestamp('start_bar', { withTimezone: true }),
    endBar: timestamp('end_bar', { withTimezone: true }),
    confidence: decimal('confidence', { precision: 4, scale: 3 }),
    priceLevel: decimal('price_level', { precision: 20, scale: 8 }),
    priceZoneLow: decimal('price_zone_low', { precision: 20, scale: 8 }),
    priceZoneHigh: decimal('price_zone_high', { precision: 20, scale: 8 }),
    strength: integer('strength'), // Test count or strength score
    isMitigated: boolean('is_mitigated').default(false),
    mitigatedAt: timestamp('mitigated_at', { withTimezone: true }),
    mitigatedByPrice: decimal('mitigated_by_price', { precision: 20, scale: 8 }),
    features: jsonb('features'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    signalIdx: index('patterns_signal_idx').on(table.signalId),
    symbolIdx: index('patterns_symbol_idx').on(table.symbol),
    typeIdx: index('patterns_type_idx').on(table.patternType),
    detectedIdx: index('patterns_detected_idx').on(table.detectedAt),
  })
);

// ==================== LESSONS ====================

export const lessons = pgTable(
  'lessons',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    lessonId: varchar('lesson_id', { length: 100 }).notNull().unique(),
    positionId: uuid('position_id').references(() => positions.id),
    signalId: uuid('signal_id').references(() => signals.id),
    sessionId: uuid('session_id').references(() => sessions.id),
    type: varchar('type', { length: 20 }).notNull(),
    category: varchar('category', { length: 50 }).notNull(),
    severity: decimal('severity', { precision: 3, scale: 2 }),
    symbol: varchar('symbol', { length: 12 }),
    timeframe: varchar('timeframe', { length: 10 }),
    marketRegime: varchar('market_regime', { length: 50 }),
    entryPrice: decimal('entry_price', { precision: 20, scale: 8 }),
    exitPrice: decimal('exit_price', { precision: 20, scale: 8 }),
    pnl: decimal('pnl', { precision: 20, scale: 4 }),
    returnPercent: decimal('return_percent', { precision: 10, scale: 4 }),
    features: jsonb('features'),
    analysis: jsonb('analysis'),
    tags: jsonb('tags'),
    notes: text('notes'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    sessionIdx: index('lessons_session_idx').on(table.sessionId),
    positionIdx: index('lessons_position_idx').on(table.positionId),
    typeIdx: index('lessons_type_idx').on(table.type),
    categoryIdx: index('lessons_category_idx').on(table.category),
    createdIdx: index('lessons_created_idx').on(table.createdAt),
  })
);

// ==================== MODEL WEIGHTS ====================

export const modelWeights = pgTable(
  'model_weights',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    modelId: varchar('model_id', { length: 100 }).notNull(),
    version: integer('version').notNull(),
    parentVersion: integer('parent_version'),
    weights: jsonb('weights').notNull(),
    accuracy: decimal('accuracy', { precision: 4, scale: 3 }),
    f1Score: decimal('f1_score', { precision: 4, scale: 3 }),
    trainingSamples: integer('training_samples'),
    validationReturn: decimal('validation_return', { precision: 10, scale: 4 }),
    validationDrawdown: decimal('validation_drawdown', { precision: 10, scale: 4 }),
    trainingRunId: varchar('training_run_id', { length: 100 }),
    trainedAt: timestamp('trained_at', { withTimezone: true }),
    isActive: boolean('is_active').default(false),
    metadata: jsonb('metadata'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    uniqueVersion: uniqueIndex('model_weights_version_idx').on(table.modelId, table.version),
    activeIdx: index('model_weights_active_idx').on(table.modelId, table.isActive),
    trainedAtIdx: index('model_weights_trained_idx').on(table.trainedAt),
  })
);

// ==================== MARKET REGIMES ====================

export const marketRegimes = pgTable(
  'market_regimes',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    symbol: varchar('symbol', { length: 12 }).notNull(),
    timeframe: varchar('timeframe', { length: 10 }).notNull(),
    regime: varchar('regime', { length: 50 }).notNull(),
    confidence: decimal('confidence', { precision: 4, scale: 3 }),
    volatility: decimal('volatility', { precision: 10, scale: 6 }),
    trendStrength: decimal('trend_strength', { precision: 4, scale: 3 }),
    features: jsonb('features'),
    validFrom: timestamp('valid_from', { withTimezone: true }).notNull(),
    validUntil: timestamp('valid_until', { withTimezone: true }),
    isCurrent: boolean('is_current').default(true),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    symbolTfIdx: index('regimes_symbol_tf_idx').on(table.symbol, table.timeframe),
    currentIdx: index('regimes_current_idx').on(table.symbol, table.timeframe, table.isCurrent),
    validFromIdx: index('regimes_valid_from_idx').on(table.validFrom),
  })
);

// ==================== PERFORMANCE SNAPSHOTS ====================

export const performanceSnapshots = pgTable(
  'performance_snapshots',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    sessionId: uuid('session_id').references(() => sessions.id),
    strategyId: varchar('strategy_id', { length: 100 }),
    snapshotAt: timestamp('snapshot_at', { withTimezone: true }).defaultNow().notNull(),
    periodStart: timestamp('period_start', { withTimezone: true }),
    periodEnd: timestamp('period_end', { withTimezone: true }),
    totalReturn: decimal('total_return', { precision: 10, scale: 4 }),
    annualizedReturn: decimal('annualized_return', { precision: 10, scale: 4 }),
    volatility: decimal('volatility', { precision: 10, scale: 4 }),
    sharpeRatio: decimal('sharpe_ratio', { precision: 10, scale: 4 }),
    sortinoRatio: decimal('sortino_ratio', { precision: 10, scale: 4 }),
    maxDrawdown: decimal('max_drawdown', { precision: 10, scale: 4 }),
    calmarRatio: decimal('calmar_ratio', { precision: 10, scale: 4 }),
    winRate: decimal('win_rate', { precision: 5, scale: 4 }),
    profitFactor: decimal('profit_factor', { precision: 10, scale: 4 }),
    totalTrades: integer('total_trades'),
    avgTradeReturn: decimal('avg_trade_return', { precision: 10, scale: 4 }),
    avgWinner: decimal('avg_winner', { precision: 10, scale: 4 }),
    avgLoser: decimal('avg_loser', { precision: 10, scale: 4 }),
    sqn: decimal('sqn', { precision: 10, scale: 4 }),
    expectancy: decimal('expectancy', { precision: 10, scale: 4 }),
    metrics: jsonb('metrics'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    sessionIdx: index('perf_session_idx').on(table.sessionId),
    timeIdx: index('perf_time_idx').on(table.snapshotAt),
    strategyIdx: index('perf_strategy_idx').on(table.strategyId),
  })
);

// ==================== EVENTS (for replay) ====================

export const events = pgTable(
  'events',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    eventId: varchar('event_id', { length: 100 }).notNull().unique(),
    eventType: varchar('event_type', { length: 100 }).notNull(),
    topic: varchar('topic', { length: 100 }).notNull(),
    sourceService: varchar('source_service', { length: 50 }).notNull(),
    occurredAt: timestamp('occurred_at', { withTimezone: true }).notNull(),
    publishedAt: timestamp('published_at', { withTimezone: true }).defaultNow().notNull(),
    traceId: varchar('trace_id', { length: 100 }),
    causationId: varchar('causation_id', { length: 100 }),
    deduplicationKey: varchar('deduplication_key', { length: 200 }),
    payload: jsonb('payload').notNull(),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    typeIdx: index('events_type_idx').on(table.eventType),
    topicIdx: index('events_topic_idx').on(table.topic),
    occurredIdx: index('events_occurred_idx').on(table.occurredAt),
    traceIdx: index('events_trace_idx').on(table.traceId),
    sourceIdx: index('events_source_idx').on(table.sourceService),
  })
);

// ==================== AUDIT LOG ====================

export const auditLog = pgTable(
  'audit_log',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    action: varchar('action', { length: 50 }).notNull(),
    entityType: varchar('entity_type', { length: 50 }).notNull(),
    entityId: varchar('entity_id', { length: 100 }).notNull(),
    sessionId: uuid('session_id'),
    performedBy: varchar('performed_by', { length: 100 }).notNull(),
    performedAt: timestamp('performed_at', { withTimezone: true }).defaultNow().notNull(),
    previousState: jsonb('previous_state'),
    newState: jsonb('new_state'),
    metadata: jsonb('metadata'),
    ipAddress: varchar('ip_address', { length: 45 }),
    userAgent: text('user_agent'),
  },
  (table) => ({
    entityIdx: index('audit_entity_idx').on(table.entityType, table.entityId),
    actionIdx: index('audit_action_idx').on(table.action),
    timeIdx: index('audit_time_idx').on(table.performedAt),
  })
);

// ==================== TYPE DEFINITIONS ====================

export type Session = typeof sessions.$inferSelect;
export type NewSession = typeof sessions.$inferInsert;

export type Signal = typeof signals.$inferSelect;
export type NewSignal = typeof signals.$inferInsert;

export type Decision = typeof decisions.$inferSelect;
export type NewDecision = typeof decisions.$inferInsert;

export type Order = typeof orders.$inferSelect;
export type NewOrder = typeof orders.$inferInsert;

export type Position = typeof positions.$inferSelect;
export type NewPosition = typeof positions.$inferInsert;

export type Trade = typeof trades.$inferSelect;
export type NewTrade = typeof trades.$inferInsert;

export type Pattern = typeof patterns.$inferSelect;
export type NewPattern = typeof patterns.$inferInsert;

export type Lesson = typeof lessons.$inferSelect;
export type NewLesson = typeof lessons.$inferInsert;

export type ModelWeight = typeof modelWeights.$inferSelect;
export type NewModelWeight = typeof modelWeights.$inferInsert;

export type MarketRegime = typeof marketRegimes.$inferSelect;
export type NewMarketRegime = typeof marketRegimes.$inferInsert;

export type PerformanceSnapshot = typeof performanceSnapshots.$inferSelect;
export type NewPerformanceSnapshot = typeof performanceSnapshots.$inferInsert;

export type Event = typeof events.$inferSelect;
export type NewEvent = typeof events.$inferInsert;

export type AuditLogEntry = typeof auditLog.$inferSelect;
export type NewAuditLogEntry = typeof auditLog.$inferInsert;
