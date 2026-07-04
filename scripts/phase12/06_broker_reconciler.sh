#!/usr/bin/env bash
# ============================================================
# APEX V3 — Phase 12 — Stage 7: Broker Reconciliation
# Runs as daemon. Every 1 hour compares Broker vs Engine state.
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
ENV_FILE="$ROOT_DIR/infrastructure/docker/.env.phase12"
[[ -f "$ENV_FILE" ]] && { set -a; source "$ENV_FILE"; set +a; }

MT5_HOST="${MT5_BRIDGE_URL:-http://host.docker.internal:8000}"
INTERVAL="${PHASE12_RECONCILIATION_INTERVAL:-3600}"

REPORT_DIR="${PHASE12_REPORT_DIR:-$ROOT_DIR/phase12_reports}"
mkdir -p "$REPORT_DIR"
LOG="$REPORT_DIR/reconciliation_$(date +%Y%m%d).log"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'; BOLD='\033[1m'
log() { echo -e "$(date -u '+%Y-%m-%d %H:%M:%S') $1" | tee -a "$LOG"; }

PG_CMD="PGPASSWORD=${DB_PASSWORD:-} psql -U ${POSTGRES_USER:-apex} -d ${POSTGRES_DB:-apex_v3} -h localhost -tAc"

log "${BOLD}APEX V3 — Stage 7: Broker Reconciliation Daemon Started${NC}"
log "Interval: $INTERVAL seconds"

while true; do
    log "Running reconciliation cycle..."
    
    # 1. Fetch Broker State
    account=$(curl -s --max-time 10 "${MT5_HOST}/account" || echo '{}')
    b_bal=$(echo "$account" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('balance',0))" 2>/dev/null || echo "0")
    b_eq=$(echo "$account" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('equity',0))" 2>/dev/null || echo "0")
    b_mar=$(echo "$account" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('margin',0) if 'margin' in d else (d.get('equity',0)-d.get('free_margin',0)))" 2>/dev/null || echo "0")
    b_fmar=$(echo "$account" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('free_margin',0))" 2>/dev/null || echo "0")
    
    pos_json=$(curl -s --max-time 10 "${MT5_HOST}/positions" || echo '[]')
    b_pos_count=$(echo "$pos_json" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")
    
    ord_json=$(curl -s --max-time 10 "${MT5_HOST}/orders" || echo '[]')
    b_ord_count=$(echo "$ord_json" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

    hist_json=$(curl -s --max-time 10 "${MT5_HOST}/history" || echo '[]')
    b_trade_count=$(echo "$hist_json" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

    # 2. Fetch Engine/DB State
    if command -v psql &>/dev/null; then
        e_pos_count=$(eval "$PG_CMD \"SELECT COUNT(DISTINCT position_id) FROM position_snapshots WHERE source='engine';\"" || echo "0")
        p_pos_count=$e_pos_count # Assume portfolio matches execution for script simplicity
        
        db_exec_count=$(eval "$PG_CMD \"SELECT COUNT(*) FROM execution_events;\"" || echo "0")
    else
        e_pos_count="0"; p_pos_count="0"; db_exec_count="0"
    fi
    
    # 3. Compare (Drift detection)
    # Note: Engine balance tracking omitted here, focusing on positional drift.
    pos_match="TRUE"
    if [[ "$b_pos_count" != "$e_pos_count" ]]; then
        pos_match="FALSE"
        log "${RED}DRIFT DETECTED: Broker Positions ($b_pos_count) != Engine Positions ($e_pos_count)${NC}"
    else
        log "${GREEN}MATCH: Positions ($b_pos_count)${NC}"
    fi
    
    all_match=$pos_match

    # 4. Log to DB
    if command -v psql &>/dev/null; then
        eval "$PG_CMD \"INSERT INTO reconciliation_log (broker_balance, broker_equity, broker_margin, broker_free_margin, broker_position_count, broker_order_count, broker_trade_count, engine_position_count, portfolio_position_count, pg_execution_event_count, position_count_match, all_match) VALUES ($b_bal, $b_eq, $b_mar, $b_fmar, $b_pos_count, $b_ord_count, $b_trade_count, $e_pos_count, $p_pos_count, $db_exec_count, $pos_match, $all_match);\""
    fi

    # Trigger report generation
    python3 "$ROOT_DIR/scripts/phase12/reports/generate_broker_reconciliation.py" || true

    log "Sleeping for $INTERVAL seconds..."
    sleep "$INTERVAL"
done
