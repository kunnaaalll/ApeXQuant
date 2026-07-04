#!/usr/bin/env bash
# ============================================================
# APEX V3 — Phase 12 — Stage 5: Chaos Testing
# Injects failures while the system is running.
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

REPORT_DIR="${PHASE12_REPORT_DIR:-$ROOT_DIR/phase12_reports}"
mkdir -p "$REPORT_DIR"
LOG="$REPORT_DIR/chaos_$(date +%Y%m%d_%H%M%S).log"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'; BOLD='\033[1m'

log() { echo -e "$(date -u '+%H:%M:%S') $1" | tee -a "$LOG"; }

log ""
log "${BOLD}╔══════════════════════════════════════════════════════════╗${NC}"
log "${BOLD}║  APEX V3 — Phase 12 — Stage 5: Chaos Injection Testing   ║${NC}"
log "${BOLD}╚══════════════════════════════════════════════════════════╝${NC}"

# Define targets
ENGINES=(
    "apex-market-data-engine"
    "apex-strategy-engine"
    "apex-risk-engine"
    "apex-portfolio-engine"
    "apex-execution-engine"
    "apex-learning-engine"
    "apex-ai-council"
)

INFRA=(
    "apex-postgres"
    "apex-redis"
    "apex-event-bus"
)

get_random() {
    local array=("$@")
    local rand=$((RANDOM % ${#array[@]}))
    echo "${array[$rand]}"
}

log "Executing chaos sequence..."

# 1. Engine Kill
TARGET_ENGINE=$(get_random "${ENGINES[@]}")
log "${RED}Injecting Failure: SIGKILL -> $TARGET_ENGINE${NC}"
docker kill "$TARGET_ENGINE" 2>/dev/null || container kill "$TARGET_ENGINE" 2>/dev/null || log "  Container tool not available"

sleep 5

# 2. Infra Restart
TARGET_INFRA=$(get_random "${INFRA[@]}")
log "${YELLOW}Injecting Failure: Restart -> $TARGET_INFRA${NC}"
docker restart "$TARGET_INFRA" 2>/dev/null || container restart "$TARGET_INFRA" 2>/dev/null || log "  Container tool not available"

sleep 10

# 3. Redis STOP/CONT
log "${RED}Injecting Failure: SIGSTOP -> apex-redis${NC}"
docker kill --signal=SIGSTOP apex-redis 2>/dev/null || container kill --signal=SIGSTOP apex-redis 2>/dev/null || true
log "  Redis frozen. Waiting 10s..."
sleep 10
log "${GREEN}Injecting Recovery: SIGCONT -> apex-redis${NC}"
docker kill --signal=SIGCONT apex-redis 2>/dev/null || container kill --signal=SIGCONT apex-redis 2>/dev/null || true

log "\n${BOLD}Chaos injection sequence complete.${NC}"
log "System must automatically recover. Check Stage 4 rules for zero-data-loss validation."
exit 0
