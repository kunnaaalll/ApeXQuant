#!/usr/bin/env bash
# APEX V3 Wave 8 - Start Shadow Campaign

set -e

echo "==============================================="
echo "   APEX V3 - LIVE SHADOW OPERATIONS CAMPAIGN"
echo "==============================================="

# Navigate to docker infra directory
cd "$(dirname "$0")/../infrastructure/docker"

# Verify .env exists
if [ ! -f .env ]; then
    echo "ERROR: .env file missing in infrastructure/docker/"
    echo "Please copy .env.template to .env and provide the required API keys."
    exit 1
fi

# Export shadow configuration path
export SHADOW_ENV_PATH="infrastructure/docker/.env"

export COMPOSE_PARALLEL_LIMIT=1

# 1. Start the core engines in shadow mode
echo "[1/4] Starting APEX V3 Platform in SHADOW MODE..."
docker-compose --profile migrate up -d --build

echo "[2/4] Waiting for services to initialize..."
sleep 15

# List of critical engines
ENGINES=("apex-event-bus" "apex-market-data-engine" "apex-strategy-engine" "apex-risk-engine" "apex-portfolio-engine" "apex-execution-engine")

echo "[3/4] Validating engine health..."
for ENGINE in "${ENGINES[@]}"; do
    if [ "$(docker inspect -f '{{.State.Running}}' "$ENGINE")" = "true" ]; then
        echo "  - $ENGINE is RUNNING"
    else
        echo "  - ERROR: $ENGINE failed to start!"
        docker logs "$ENGINE" --tail 20
        exit 1
    fi
done

echo "[4/4] Validating database and observability..."
for INFRA in "apex-postgres" "apex-redis" "apex-prometheus" "apex-grafana" "apex-loki" "apex-jaeger"; do
    if [ "$(docker inspect -f '{{.State.Running}}' "$INFRA")" = "true" ]; then
        echo "  - $INFRA is RUNNING"
    else
        echo "  - ERROR: $INFRA failed to start!"
        exit 1
    fi
done

echo "==============================================="
echo " SHADOW CAMPAIGN SUCCESSFULLY DEPLOYED"
echo "==============================================="
echo "Grafana: http://localhost:3002"
echo "Prometheus: http://localhost:9090"
echo "Jaeger: http://localhost:16686"
echo ""
echo "System is now ingesting feeds from Binance Testnet and MT5 Demo."
echo "No live capital is at risk. Max risk per trade enforced at 0.5%."
