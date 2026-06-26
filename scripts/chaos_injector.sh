#!/usr/bin/env bash
# APEX V3 Wave 8 - Chaos Injector
# Injects random failures to validate 100% deterministic recovery

set -e

echo "==============================================="
echo "   APEX V3 - CHAOS INJECTION UTILITY"
echo "==============================================="

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

# Function to get random element from array
get_random() {
    local array=("$@")
    local rand=$((RANDOM % ${#array[@]}))
    echo "${array[$rand]}"
}

# 1. Engine Crash (Kill)
TARGET_ENGINE=$(get_random "${ENGINES[@]}")
echo "[CHAOS] Killing Engine: $TARGET_ENGINE"
docker kill "$TARGET_ENGINE" || true

# Wait 5 seconds
sleep 5

# 2. Infra Restart
TARGET_INFRA=$(get_random "${INFRA[@]}")
echo "[CHAOS] Restarting Infrastructure: $TARGET_INFRA"
docker restart "$TARGET_INFRA" || true

echo "[CHAOS] Injections complete. The system should auto-recover via Docker restart policies."
echo "Monitor the Grafana Infrastructure Dashboard for Recovery Success metrics."
