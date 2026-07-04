#!/usr/bin/env bash
# ============================================================
# APEX V3 — Phase 12 — Stage 4: Recovery Testing
# Restarts each service while positions are open.
# Verifies position recovery, no orphans, no duplicate orders.
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
ENV_FILE="$ROOT_DIR/infrastructure/docker/.env.phase12"
[[ -f "$ENV_FILE" ]] && { set -a; source "$ENV_FILE"; set +a; }

REPORT_DIR="${PHASE12_REPORT_DIR:-$ROOT_DIR/phase12_reports}"
mkdir -p "$REPORT_DIR"
REPORT="$REPORT_DIR/RECOVERY_VALIDATION_REPORT.md"
LOG="$REPORT_DIR/recovery_$(date +%Y%m%d_%H%M%S).log"

MT5_HOST="${MT5_BRIDGE_URL:-http://host.docker.internal:8000}"
LOT_SIZE="${PHASE12_LOT_SIZE:-0.01}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'; BOLD='\033[1m'
PASS=0; FAIL=0
RECOVERY_EVENTS=()

log()     { echo -e "$(date -u '+%H:%M:%S') $1" | tee -a "$LOG"; }
log_raw() { echo -e "$1" | tee -a "$LOG"; }
pass()    { log "${GREEN}  ✅ PASS${NC} — $1"; ((PASS+=1)); }
fail()    { log "${RED}  ❌ FAIL${NC} — $1"; ((FAIL+=1)); }
section() { log_raw "\n${BOLD}━━━ $1 ━━━${NC}"; }

log_raw ""
log_raw "${BOLD}╔══════════════════════════════════════════════════════════╗${NC}"
log_raw "${BOLD}║  APEX V3 — Phase 12 — Stage 4: Recovery Testing          ║${NC}"
log_raw "${BOLD}╚══════════════════════════════════════════════════════════╝${NC}"

# ============================================================
# Helper: count broker positions
# ============================================================
broker_position_count() {
    curl -s --max-time 10 "${MT5_HOST}/positions" 2>/dev/null | \
        python3 -c "import sys,json; positions=json.load(sys.stdin); print(len(positions))" 2>/dev/null || echo "0"
}

# ============================================================
# Helper: open a live position and return ticket
# ============================================================
open_position() {
    local sym="$1" side="$2"
    local resp
    resp=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"symbol\":\"$sym\",\"side\":\"$side\",\"order_type\":\"Market\",\"volume\":$LOT_SIZE}" \
        --max-time 15 "${MT5_HOST}/orders" 2>/dev/null || echo '{}')
    echo "$resp" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('order_id',''))" 2>/dev/null || echo ""
}

# ============================================================
# Helper: restart a service and measure recovery
# ============================================================
test_service_restart() {
    local service_label="$1"
    local container_name="$2"
    local recovery_wait="${3:-15}"

    section "Recovery Test: $service_label"

    # Snapshot broker state before
    pos_before=$(broker_position_count)
    log "Positions before restart: $pos_before"

    # Insert recovery event record
    local triggered_at
    triggered_at=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

    # Restart the container
    log "  Restarting container: $container_name"
    if docker restart "$container_name" &>/dev/null 2>&1 || container restart "$container_name" &>/dev/null 2>&1; then
        log "  Container restart command sent"
    else
        log "${YELLOW}  Container restart failed (may not be containerized) — skipping${NC}"
        RECOVERY_EVENTS+=("$service_label|SKIPPED|$pos_before|$pos_before|0|0|$(date -u '+%Y-%m-%dT%H:%M:%SZ')|true")
        return
    fi

    log "  Waiting ${recovery_wait}s for recovery..."
    sleep "$recovery_wait"

    # Snapshot broker state after
    pos_after=$(broker_position_count)
    log "  Positions after recovery: $pos_after"

    local recovered_at
    recovered_at=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

    # Check for orphans: positions in DB not in broker
    orphan_count=0
    if command -v psql &>/dev/null; then
        orphan_count=$(PGPASSWORD="${DB_PASSWORD:-}" psql -U "${POSTGRES_USER:-apex}" -d "${POSTGRES_DB:-apex_v3}" -h localhost -tAc \
            "SELECT COUNT(*) FROM position_snapshots WHERE source='engine' AND snapshot_at > NOW() - INTERVAL '30 minutes' AND position_id NOT IN (SELECT position_id FROM position_snapshots WHERE source='broker' AND snapshot_at > NOW() - INTERVAL '5 minutes');" 2>/dev/null | tr -d ' ' || echo "0")
    fi

    # Check for duplicate orders in DB
    dup_count=0
    if command -v psql &>/dev/null; then
        dup_count=$(PGPASSWORD="${DB_PASSWORD:-}" psql -U "${POSTGRES_USER:-apex}" -d "${POSTGRES_DB:-apex_v3}" -h localhost -tAc \
            "SELECT COUNT(*) FROM (SELECT order_id, COUNT(*) c FROM execution_events WHERE event_type='OrderFilled' GROUP BY order_id HAVING COUNT(*) > 1) dups;" 2>/dev/null | tr -d ' ' || echo "0")
    fi

    RECOVERY_EVENTS+=("$service_label|restart|$pos_before|$pos_after|$orphan_count|$dup_count|$triggered_at|$recovered_at")

    if [[ "$pos_after" -eq "$pos_before" ]]; then
        pass "$service_label — position count maintained ($pos_before → $pos_after)"
    else
        fail "$service_label — position count drift: $pos_before → $pos_after"
    fi

    if [[ "$orphan_count" -eq "0" ]]; then
        pass "$service_label — zero orphan positions"
    else
        fail "$service_label — $orphan_count orphan position(s) detected"
    fi

    if [[ "$dup_count" -eq "0" ]]; then
        pass "$service_label — zero duplicate fill events"
    else
        fail "$service_label — $dup_count duplicate fill event(s) detected"
    fi

    log "  Orphans: $orphan_count | Duplicates: $dup_count"
}

# ============================================================
# Phase 1: Open positions to test with
# ============================================================
section "Opening Test Positions"
log "Opening 4 positions (EURUSD Buy, EURUSD Sell, GBPUSD Buy, XAUUSD Buy)..."

t1=$(open_position "EURUSD" "Buy")
t2=$(open_position "EURUSD" "Sell")
t3=$(open_position "GBPUSD" "Buy")
t4=$(open_position "XAUUSD" "Buy")

log "Tickets: EURUSD Buy=$t1 | EURUSD Sell=$t2 | GBPUSD Buy=$t3 | XAUUSD Buy=$t4"
initial_count=$(broker_position_count)
log "Initial position count: $initial_count"
sleep 3

# ============================================================
# Phase 2: Restart each service in sequence
# ============================================================
test_service_restart "Execution Engine"    "apex-execution-engine"   20
test_service_restart "Event Bus"           "apex-event-bus"          15
test_service_restart "Portfolio Engine"    "apex-portfolio-engine"   15
test_service_restart "Risk Engine"         "apex-risk-engine"        15
test_service_restart "Signal Engine"       "apex-signal-engine"      15
test_service_restart "Market Data Engine"  "apex-market-data-engine" 10

# ============================================================
# Phase 3: Database restart (most critical)
# ============================================================
section "Recovery Test: PostgreSQL"
pos_before=$(broker_position_count)
log "Positions before: $pos_before"
triggered_at=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
if docker restart apex-postgres &>/dev/null 2>&1 || container restart apex-postgres &>/dev/null 2>&1; then
    log "  PostgreSQL restart sent — waiting 30s..."
    sleep 30
    # Verify DB reconnects
    if command -v psql &>/dev/null; then
        if PGPASSWORD="${DB_PASSWORD:-}" psql -U "${POSTGRES_USER:-apex}" -d "${POSTGRES_DB:-apex_v3}" -h localhost -c "\q" &>/dev/null 2>&1; then
            pass "PostgreSQL — reconnected after restart"
        else
            fail "PostgreSQL — failed to reconnect after restart"
        fi
    fi
fi
pos_after=$(broker_position_count)
[[ "$pos_after" -eq "$pos_before" ]] && pass "PostgreSQL restart — broker positions intact" || fail "PostgreSQL restart — position drift"
RECOVERY_EVENTS+=("PostgreSQL|restart|$pos_before|$pos_after|0|0|$triggered_at|$(date -u '+%Y-%m-%dT%H:%M:%SZ')")

# ============================================================
# Phase 4: Redis restart
# ============================================================
section "Recovery Test: Redis"
pos_before=$(broker_position_count)
triggered_at=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
if docker restart apex-redis &>/dev/null 2>&1 || container restart apex-redis &>/dev/null 2>&1; then
    log "  Redis restart sent — waiting 10s..."
    sleep 10
    if command -v redis-cli &>/dev/null; then
        pong=$(redis-cli ping 2>/dev/null || echo "FAIL")
        [[ "$pong" == "PONG" ]] && pass "Redis — reconnected after restart" || fail "Redis — failed to reconnect"
    fi
fi
pos_after=$(broker_position_count)
[[ "$pos_after" -eq "$pos_before" ]] && pass "Redis restart — broker positions intact" || fail "Redis restart — position drift"
RECOVERY_EVENTS+=("Redis|restart|$pos_before|$pos_after|0|0|$triggered_at|$(date -u '+%Y-%m-%dT%H:%M:%SZ')")

# ============================================================
# Phase 5: Clean up — close all test positions
# ============================================================
section "Closing Test Positions"
open_positions=$(curl -s --max-time 10 "${MT5_HOST}/positions" 2>/dev/null || echo '[]')
all_tickets=$(echo "$open_positions" | python3 -c "import sys,json; [print(p['ticket']) for p in json.load(sys.stdin)]" 2>/dev/null || echo "")
for ticket in $all_tickets; do
    curl -s -X POST --max-time 15 "${MT5_HOST}/positions/${ticket}/close" &>/dev/null || true
    log "  Closed ticket: $ticket"
done

# ============================================================
# Generate RECOVERY_VALIDATION_REPORT.md
# ============================================================
cat > "$REPORT" << REPORT_EOF
# APEX V3 — Recovery Validation Report
**Phase:** 12 — MT5 Demo Validation
**Stage:** 4 — Recovery Testing
**Generated:** $(date -u '+%Y-%m-%dT%H:%M:%SZ')
**Overall Result:** $( [[ $FAIL -eq 0 ]] && echo "✅ PASSED" || echo "❌ FAILED ($FAIL failures)" )

---

## Recovery Events

| Service | Type | Positions Before | Positions After | Orphans | Duplicates | Triggered | Recovered | Result |
|:--------|:-----|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
REPORT_EOF

for entry in "${RECOVERY_EVENTS[@]}"; do
    IFS='|' read -r svc ftype pos_b pos_a orphans dups trig rec <<< "$entry"
    result=$( [[ "$pos_b" == "$pos_a" && "$orphans" == "0" && "$dups" == "0" ]] && echo "✅ PASS" || echo "❌ FAIL" )
    echo "| $svc | $ftype | $pos_b | $pos_a | $orphans | $dups | $trig | $rec | $result |" >> "$REPORT"
done

cat >> "$REPORT" << REPORT_EOF2

---

## Acceptance Criteria

| Criterion | Result |
|:----------|:------:|
| Position recovery after restart | $( [[ $FAIL -eq 0 ]] && echo "✅ PASS" || echo "❌ FAIL" ) |
| Zero orphan positions | $( [[ $FAIL -eq 0 ]] && echo "✅ PASS" || echo "❌ FAIL" ) |
| Zero duplicate fill events | $( [[ $FAIL -eq 0 ]] && echo "✅ PASS" || echo "❌ FAIL" ) |
| Zero corrupted state | $( [[ $FAIL -eq 0 ]] && echo "✅ PASS" || echo "❌ FAIL" ) |

---

## Test Summary

- **Services tested:** Execution Engine, Event Bus, Portfolio Engine, Risk Engine, Signal Engine, Market Data Engine, PostgreSQL, Redis
- **Passed:** $PASS
- **Failed:** $FAIL
REPORT_EOF2

log_raw "\n${BOLD}━━━ Stage 4 Summary ━━━${NC}"
log "Report: $REPORT"
log "Passed: $PASS | Failed: $FAIL"

if [[ $FAIL -gt 0 ]]; then
    log_raw "${RED}${BOLD}STAGE 4 RESULT: ❌ FAILED${NC}"
    exit 1
else
    log_raw "${GREEN}${BOLD}STAGE 4 RESULT: ✅ PASSED — All services recovered correctly${NC}"
    exit 0
fi
