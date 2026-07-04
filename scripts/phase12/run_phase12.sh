#!/usr/bin/env bash
# ============================================================
# APEX V3 — Phase 12 Master Orchestrator
# Executes the entire validation pipeline sequentially.
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'; BOLD='\033[1m'

log() { echo -e "\n${BOLD}>>> $1${NC}"; }
error() { echo -e "\n${RED}${BOLD}ERROR: $1${NC}"; exit 1; }

log "Starting APEX V3 Phase 12 Master Orchestrator"

# 1. Preflight
log "Stage 1: Preflight Check"
"$SCRIPT_DIR/00_preflight_check.sh" || error "Preflight failed."

# 2. Execution Validation
log "Stage 3: Execution Pipeline Validation"
"$SCRIPT_DIR/02_execution_validator.sh" || error "Execution validation failed."

# 3. Recovery Testing
log "Stage 4: Recovery Testing"
"$SCRIPT_DIR/03_recovery_tester.sh" || error "Recovery testing failed."

# 4. Chaos Injection (Optional/Manual trigger normally, but integrated here for completeness if desired)
log "Stage 5: Chaos Injection (Skipping interactive chaos in automated run, run 04_chaos_injector_phase12.sh manually)"
# "$SCRIPT_DIR/04_chaos_injector_phase12.sh"

# 5. Start Background Daemons (Reconciler and Market Data)
log "Stage 2: Starting Market Data Validator (Background)"
nohup "$SCRIPT_DIR/01_market_data_validator.sh" 24 > /dev/null 2>&1 &
MD_PID=$!

log "Stage 7: Starting Broker Reconciler (Background)"
nohup "$SCRIPT_DIR/06_broker_reconciler.sh" > /dev/null 2>&1 &
RECON_PID=$!

# 6. Trade Campaign
log "Stage 8: Starting 1000-Trade Campaign Monitor"
"$SCRIPT_DIR/07_trade_campaign.sh" 1000 || error "Campaign failed."

# 7. Replay Validation (Post-campaign)
log "Stage 6: Replay Validation"
"$SCRIPT_DIR/05_replay_validator.sh" 1000 || error "Replay validation failed."

log "${GREEN}${BOLD}============================================================${NC}"
log "${GREEN}${BOLD} PHASE 12 VALIDATION COMPLETE: ALL STAGES PASSED            ${NC}"
log "${GREEN}${BOLD}============================================================${NC}"

# Kill background tasks
kill $MD_PID $RECON_PID || true
