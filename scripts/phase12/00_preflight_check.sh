#!/usr/bin/env bash
# ============================================================
# APEX V3 — Phase 12 — Stage 1: Infrastructure Preflight Check
# Verifies every service is online before starting the campaign.
# Exit 0 = all critical services UP. Exit 1 = any critical failure.
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Load environment
ENV_FILE="$ROOT_DIR/infrastructure/docker/.env.phase12"
if [[ -f "$ENV_FILE" ]]; then
    set -a; source "$ENV_FILE"; set +a
fi

REPORT_DIR="${PHASE12_REPORT_DIR:-$ROOT_DIR/phase12_reports}"
mkdir -p "$REPORT_DIR"
LOG="$REPORT_DIR/preflight_$(date +%Y%m%d_%H%M%S).log"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'; BOLD='\033[1m'

PASS=0; FAIL=0
declare -a FAILURES=()

log() { echo -e "$1" | tee -a "$LOG"; }
pass() { log "${GREEN}  ✅  PASS${NC} — $1"; ((PASS+=1)); }
fail() { log "${RED}  ❌  FAIL${NC} — $1"; ((FAIL+=1)); FAILURES+=("$1"); }
warn() { log "${YELLOW}  ⚠️   WARN${NC} — $1"; }
section() { log "\n${BOLD}━━━ $1 ━━━${NC}"; }

# TCP port probe (no netcat dependency)
check_port() {
    local host="$1" port="$2" label="$3" timeout_sec="${4:-3}"
    if python3 -c "
import socket, sys
s = socket.socket()
s.settimeout($timeout_sec)
try:
    s.connect(('$host', $port)); s.close(); sys.exit(0)
except: sys.exit(1)
" 2>/dev/null; then
        pass "$label (${host}:${port})"
    else
        fail "$label (${host}:${port}) — port not reachable"
    fi
}

# HTTP health probe
check_http() {
    local url="$1" label="$2" expected_status="${3:-200}"
    local status
    status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$url" 2>/dev/null || echo "000")
    if [[ "$status" == "$expected_status" ]]; then
        pass "$label — HTTP $status"
    else
        fail "$label — HTTP $status (expected $expected_status) at $url"
    fi
}

# ============================================================
log ""
log "${BOLD}╔════════════════════════════════════════════════════════════╗${NC}"
log "${BOLD}║     APEX V3 — Phase 12 — Stage 1: Infrastructure Preflight  ║${NC}"
log "${BOLD}╚════════════════════════════════════════════════════════════╝${NC}"
log "Timestamp : $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
log "Log       : $LOG"

# ============================================================
section "Core Infrastructure"
check_port "localhost" "5432" "PostgreSQL"
check_port "localhost" "6379" "Redis"
check_port "localhost" "${EVENT_BUS_GRPC_PORT:-8080}" "Event Bus (gRPC)"

# ============================================================
section "Rust Engine Services"
check_port "localhost" "8081" "Signal Engine"
check_port "localhost" "8082" "Risk Engine"
check_port "localhost" "8083" "Execution Engine"
check_port "localhost" "8084" "Position Engine"
check_port "localhost" "8085" "Portfolio Engine"
check_port "localhost" "8086" "Analytics Engine"
check_port "localhost" "8087" "Learning Engine"
check_port "localhost" "8088" "Backtester"
check_port "localhost" "8089" "Market Data Engine"

# ============================================================
section "Node.js Applications"
check_http "http://localhost:3000/health" "API Gateway"
check_http "http://localhost:3001" "Dashboard" "200"
check_port "localhost" "3002" "Orchestrator"
check_port "localhost" "3003" "AI Council"

# ============================================================
section "MT5 Bridge"
MT5_HOST="${MT5_BRIDGE_URL:-http://host.docker.internal:8000}"
mt5_ping=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "${MT5_HOST}/ping" 2>/dev/null || echo "000")
if [[ "$mt5_ping" == "200" ]]; then
    pass "MT5 Bridge /ping — HTTP 200"
    # Full health check
    health_resp=$(curl -s --max-time 10 "${MT5_HOST}/health/full" 2>/dev/null || echo '{}')
    connected=$(echo "$health_resp" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d.get('terminal_info') or {}; print(t.get('connected','false'))" 2>/dev/null || echo "false")
    trade_allowed=$(echo "$health_resp" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d.get('terminal_info') or {}; print(t.get('trade_allowed','false'))" 2>/dev/null || echo "false")
    mt5_status=$(echo "$health_resp" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('status','unknown'))" 2>/dev/null || echo "unknown")
    log "  MT5 Bridge status   : $mt5_status"
    log "  Terminal connected  : $connected"
    log "  Trade allowed       : $trade_allowed"
    [[ "$connected" == "True" ]] && pass "MT5 Terminal Connected" || fail "MT5 Terminal NOT connected — start MetaTrader 5"
    [[ "$trade_allowed" == "True" ]] && pass "MT5 Trade Allowed" || warn "MT5 Trade NOT allowed — check account permissions"
else
    fail "MT5 Bridge /ping — HTTP $mt5_ping — Is mt5_bridge.py running on Windows host?"
fi

# ============================================================
section "Observability Stack"
check_port "localhost" "9090" "Prometheus"
check_port "localhost" "3004" "Grafana"
check_port "localhost" "16686" "Jaeger"

# ============================================================
section "Database Connectivity"
if command -v psql &>/dev/null; then
    if PGPASSWORD="${DB_PASSWORD:-}" psql -U "${POSTGRES_USER:-apex}" -d "${POSTGRES_DB:-apex_v3}" -h localhost -c "\dt" &>/dev/null; then
        table_count=$(PGPASSWORD="${DB_PASSWORD:-}" psql -U "${POSTGRES_USER:-apex}" -d "${POSTGRES_DB:-apex_v3}" -h localhost -tAc "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public';" 2>/dev/null || echo "0")
        pass "PostgreSQL — connected — $table_count tables in public schema"
        if [[ "$table_count" -lt 10 ]]; then
            warn "Only $table_count tables found — run: psql -U apex -d apex_v3 -f infrastructure/docker/init-scripts/001_apex_v3_schema.sql"
        fi
    else
        fail "PostgreSQL — cannot connect with provided credentials"
    fi
else
    warn "psql not found — skipping direct DB check (relying on port probe)"
fi

# Redis check
if command -v redis-cli &>/dev/null; then
    pong=$(redis-cli ping 2>/dev/null || echo "FAIL")
    [[ "$pong" == "PONG" ]] && pass "Redis — PONG received" || fail "Redis — no PONG"
else
    warn "redis-cli not found — relying on port probe"
fi

# ============================================================
section "Summary"
log ""
log "  ${GREEN}Passed  : $PASS${NC}"
log "  ${RED}Failed  : $FAIL${NC}"
log ""

if [[ $FAIL -gt 0 ]]; then
    log "${RED}${BOLD}STAGE 1 RESULT: ❌ FAILED — $FAIL service(s) not ready${NC}"
    log ""
    log "Failed services:"
    for f in "${FAILURES[@]}"; do log "  • $f"; done
    log ""
    log "Resolve all failures before proceeding to Stage 2."
    exit 1
else
    log "${GREEN}${BOLD}STAGE 1 RESULT: ✅ PASSED — All $PASS services online${NC}"
    log ""
    log "Infrastructure is ready. Proceed to Stage 2 (Market Data Validation)."
    exit 0
fi
