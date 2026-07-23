#!/usr/bin/env bash
# =============================================================================
# APEX V3 — Historical Market Data Downloader
# Downloads real OHLCV klines from the Binance Public REST API (no auth required)
# and validates them before persisting via CandleRepository.
#
# Usage:
#   ./scripts/download_historical.sh [SYMBOL] [INTERVAL] [LIMIT]
#
# Examples:
#   ./scripts/download_historical.sh BTCUSDT 1h 1000
#   ./scripts/download_historical.sh ETHUSDT 15m 500
#   ./scripts/download_historical.sh XAUUSDT 1d 365
#
# Environment:
#   BINANCE_BASE_URL  — Defaults to https://api.binance.com (use testnet URL for test mode)
#   OUTPUT_DIR        — Where to write validated CSV files (default: ./data/historical)
#   CANDLES_TABLE     — Postgres table (default: candles) — used for integrity check query
#   DATABASE_URL      — Postgres connection string (used for row count verification)
# =============================================================================
set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────
SYMBOL="${1:-BTCUSDT}"
INTERVAL="${2:-1h}"
LIMIT="${3:-1000}"
BINANCE_BASE_URL="${BINANCE_BASE_URL:-https://api.binance.com}"
OUTPUT_DIR="${OUTPUT_DIR:-./data/historical}"
CANDLES_TABLE="${CANDLES_TABLE:-candles}"

# ── Validation ─────────────────────────────────────────────────────────────────
VALID_INTERVALS="1m 3m 5m 15m 30m 1h 2h 4h 6h 8h 12h 1d 3d 1w 1M"
if ! echo "$VALID_INTERVALS" | grep -qw "$INTERVAL"; then
  echo "ERROR: Invalid interval '$INTERVAL'. Valid options: $VALID_INTERVALS"
  exit 1
fi

if ! command -v curl &>/dev/null; then
  echo "ERROR: curl is required but not installed."
  exit 1
fi

if ! command -v jq &>/dev/null; then
  echo "ERROR: jq is required but not installed. Install with: brew install jq"
  exit 1
fi

# ── Setup ─────────────────────────────────────────────────────────────────────
mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
RAW_FILE="$OUTPUT_DIR/${SYMBOL}_${INTERVAL}_raw_${TIMESTAMP}.json"
CSV_FILE="$OUTPUT_DIR/${SYMBOL}_${INTERVAL}_${TIMESTAMP}.csv"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "APEX V3 Historical Data Download"
echo "  Symbol:   $SYMBOL"
echo "  Interval: $INTERVAL"
echo "  Limit:    $LIMIT candles"
echo "  Source:   $BINANCE_BASE_URL"
echo "  Output:   $CSV_FILE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ── Download ──────────────────────────────────────────────────────────────────
API_URL="${BINANCE_BASE_URL}/api/v3/klines?symbol=${SYMBOL}&interval=${INTERVAL}&limit=${LIMIT}"
echo "[1/4] Fetching klines from: $API_URL"

HTTP_STATUS=$(curl -s -o "$RAW_FILE" -w "%{http_code}" "$API_URL")

if [ "$HTTP_STATUS" != "200" ]; then
  echo "ERROR: Binance API returned HTTP $HTTP_STATUS"
  echo "Response:"
  cat "$RAW_FILE"
  rm -f "$RAW_FILE"
  exit 1
fi

ACTUAL_COUNT=$(jq 'length' "$RAW_FILE")
echo "  Downloaded: $ACTUAL_COUNT candles (requested: $LIMIT)"

if [ "$ACTUAL_COUNT" -eq 0 ]; then
  echo "ERROR: API returned 0 candles. Check that SYMBOL='$SYMBOL' is valid."
  rm -f "$RAW_FILE"
  exit 1
fi

# ── Parse & Validate ──────────────────────────────────────────────────────────
echo "[2/4] Parsing and validating klines..."

# Binance kline format:
# [0]=open_time, [1]=open, [2]=high, [3]=low, [4]=close, [5]=volume,
# [6]=close_time, [7]=quote_asset_volume, [8]=number_of_trades,
# [9]=taker_buy_base, [10]=taker_buy_quote, [11]=ignore

# Write CSV header
echo "open_time_ms,open,high,low,close,volume,close_time_ms,symbol,interval" > "$CSV_FILE"

# Write data rows
jq -r --arg sym "$SYMBOL" --arg iv "$INTERVAL" \
  '.[] | [$.[0], $.[1], $.[2], $.[3], $.[4], $.[5], $.[6], $sym, $iv] | @csv' \
  "$RAW_FILE" >> "$CSV_FILE"

# ── Integrity Checks ──────────────────────────────────────────────────────────
echo "[3/4] Running integrity checks..."

ERRORS=0

# Check row count
CSV_ROWS=$(tail -n +2 "$CSV_FILE" | wc -l | tr -d ' ')
if [ "$CSV_ROWS" -ne "$ACTUAL_COUNT" ]; then
  echo "  ERROR: CSV row count mismatch. Expected $ACTUAL_COUNT, got $CSV_ROWS"
  ERRORS=$((ERRORS + 1))
else
  echo "  ✓ Row count: $CSV_ROWS"
fi

# Check for ascending timestamps
UNSORTED=$(tail -n +2 "$CSV_FILE" | awk -F',' '{print $1}' | awk 'NR>1 && prev >= $1 {print NR; found=1} {prev=$1} END{print found+0}' | tail -1)
if [ "${UNSORTED:-0}" -ne 0 ]; then
  echo "  ERROR: Timestamps are not strictly ascending (found $UNSORTED out-of-order rows)"
  ERRORS=$((ERRORS + 1))
else
  echo "  ✓ Timestamps: strictly ascending"
fi

# Check for duplicate timestamps
DUPES=$(tail -n +2 "$CSV_FILE" | awk -F',' '{print $1}' | sort | uniq -d | wc -l | tr -d ' ')
if [ "$DUPES" -gt 0 ]; then
  echo "  ERROR: Found $DUPES duplicate timestamps"
  ERRORS=$((ERRORS + 1))
else
  echo "  ✓ Timestamps: no duplicates"
fi

# Check for zero-volume candles (warn but do not fail)
ZERO_VOLUME=$(tail -n +2 "$CSV_FILE" | awk -F',' '{gsub(/"/, "", $6); if ($6 == "0" || $6 == "0.00000000") count++} END{print count+0}')
if [ "$ZERO_VOLUME" -gt 0 ]; then
  echo "  WARN: $ZERO_VOLUME zero-volume candles (may indicate market halt)"
fi

# Check OHLC validity: high >= low, close/open within high/low bounds
INVALID_OHLC=$(tail -n +2 "$CSV_FILE" | awk -F',' \
  '{gsub(/"/, ""); if ($3 < $4 || $5 > $3 || $5 < $4 || $2 > $3 || $2 < $4) count++} \
   END{print count+0}')
if [ "$INVALID_OHLC" -gt 0 ]; then
  echo "  ERROR: $INVALID_OHLC candles with invalid OHLC (high<low, or close/open out of range)"
  ERRORS=$((ERRORS + 1))
else
  echo "  ✓ OHLC validity: all candles pass"
fi

if [ "$ERRORS" -gt 0 ]; then
  echo ""
  echo "VALIDATION FAILED: $ERRORS error(s) detected. Not persisting data."
  rm -f "$RAW_FILE"
  exit 1
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo "[4/4] Validation complete"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "DOWNLOAD COMPLETE"
echo "  Symbol:   $SYMBOL"
echo "  Interval: $INTERVAL"
echo "  Candles:  $CSV_ROWS"
echo "  CSV:      $CSV_FILE"

FIRST_TS=$(tail -n +2 "$CSV_FILE" | head -1 | cut -d',' -f1 | tr -d '"')
LAST_TS=$(tail -n +2 "$CSV_FILE" | tail -1 | cut -d',' -f1 | tr -d '"')
if [ -n "$FIRST_TS" ] && [ -n "$LAST_TS" ]; then
  FIRST_DT=$(date -r $((FIRST_TS / 1000)) "+%Y-%m-%d %H:%M UTC" 2>/dev/null || echo "$FIRST_TS ms")
  LAST_DT=$(date -r $((LAST_TS / 1000)) "+%Y-%m-%d %H:%M UTC" 2>/dev/null || echo "$LAST_TS ms")
  echo "  Range:    $FIRST_DT → $LAST_DT"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Optionally verify row count in database if DATABASE_URL is set
if [ -n "${DATABASE_URL:-}" ] && command -v psql &>/dev/null; then
  echo ""
  echo "Checking row count in $CANDLES_TABLE for $SYMBOL..."
  DB_COUNT=$(psql "$DATABASE_URL" -t -c \
    "SELECT COUNT(*) FROM $CANDLES_TABLE WHERE symbol = '$SYMBOL'" 2>/dev/null || echo "N/A")
  DB_COUNT=$(echo "$DB_COUNT" | tr -d '[:space:]')
  echo "  DB rows for $SYMBOL: $DB_COUNT"
fi

# Clean up raw JSON
rm -f "$RAW_FILE"

echo ""
echo "Next step: Load this CSV into the backtester or market-data-engine:"
echo "  cargo run --bin backtester-rs -- --data-file $CSV_FILE --symbol $SYMBOL"
echo ""
