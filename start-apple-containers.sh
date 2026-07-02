#!/bin/bash

# start-apple-containers.sh
# This script starts the APEX V3 stack using Apple's native `container` CLI.
# Run this from the root of the apex-v3 project workspace.

set -e

echo "🚀 Starting APEX V3 using Apple Containerization..."

# 1. Load environment variables
if [ -f "infrastructure/docker/.env" ]; then
    echo "Loading .env file..."
    export $(grep -v '^#' infrastructure/docker/.env | xargs)
else
    echo "⚠️  No .env file found in infrastructure/docker/.env"
fi

# 2. Create local directories for volumes (since named volumes might not be supported natively the same way)
mkdir -p infrastructure/docker/volumes/{postgres,redis,models,prometheus,grafana,loki} infrastructure/docker/init-scripts

# 3. Create Network (if supported, otherwise we rely on port mapping)
# container network create apex-network || true

echo "🧹 Cleaning up old containers..."
container rm -f $(container ls -a -q) 2>/dev/null || true

echo "📦 Starting Core Infrastructure..."

# Postgres
container run -d --name apex-postgres \
    -p 5432:5432 \
    -e POSTGRES_USER=apex \
    -e POSTGRES_PASSWORD="${DB_PASSWORD}" \
    -e POSTGRES_DB=apex_v3 \
    -v "$(pwd)/infrastructure/docker/volumes/postgres:/var/lib/postgresql/data" \
    -v "$(pwd)/infrastructure/docker/init-scripts:/docker-entrypoint-initdb.d" \
    index.docker.io/library/postgres:16-alpine

# Redis
container run -d --name apex-redis \
    -p 6379:6379 \
    -v "$(pwd)/infrastructure/docker/volumes/redis:/data" \
    index.docker.io/library/redis:7-alpine redis-server --appendonly yes --maxmemory 2gb --maxmemory-policy allkeys-lru

echo "⏳ Waiting for databases to initialize..."
sleep 5

echo "🛠  Building and Starting Rust Services..."
RUST_SERVICES=(
    "event-bus:8080:event-bus-rs"
    "signal-engine:8081:signal-engine-rs"
    "risk-engine:8082:risk-engine-rs"
    "execution-engine:8083:execution-engine-rs"
    "position-engine:8084:position-engine-rs"
    "portfolio-engine:8085:portfolio-engine-rs"
    "analytics-engine:8086:analytics-engine-rs"
    "learning-engine:8087:learning-engine-rs"
    "backtester:8088:backtester-rs"
)

for svc in "${RUST_SERVICES[@]}"; do
    IFS=":" read -r name port bin_name <<< "${svc}"
    echo "Building ${name}..."
    container build -t "apex/${name}" -f infrastructure/docker/Dockerfile.rust --build-arg SERVICE="${bin_name}" .
    
    echo "Starting ${name}..."
    container run -d --name "apex-${name}" \
        -p "${port}:${port}" \
        -e RUST_LOG=info \
        -e SERVICE_NAME="${name}" \
        -e SERVICE_PORT="${port}" \
        -e REDIS_URL=redis://localhost:6379 \
        -e DATABASE_URL=postgres://apex:${DB_PASSWORD}@localhost:5432/apex_v3 \
        -v "$(pwd)/infrastructure/docker/volumes/models:/models" \
        "apex/${name}"
done

echo "🛠  Building and Starting TypeScript Services..."
TS_SERVICES=(
    "api:3000:api"
    "dashboard:3001:dashboard"
    "orchestrator:3002:orchestrator"
    "ai-council:3003:ai-council"
)

for svc in "${TS_SERVICES[@]}"; do
    IFS=":" read -r name port app_name <<< "${svc}"
    echo "Building ${name}..."
    container build -t "apex/${name}" -f infrastructure/docker/Dockerfile.typescript --build-arg APP="${app_name}" .
    
    echo "Starting ${name}..."
    container run -d --name "apex-${name}" \
        -p "${port}:${port}" \
        -e NODE_ENV=production \
        -e PORT="${port}" \
        -e SERVICE_NAME="${name}" \
        "apex/${name}"
done

echo "📊 Starting Observability Stack..."
# Prometheus
container run -d --name apex-prometheus \
    -p 9090:9090 \
    -v "$(pwd)/infrastructure/monitoring/prometheus:/etc/prometheus" \
    -v "$(pwd)/infrastructure/docker/volumes/prometheus:/prometheus" \
    index.docker.io/prom/prometheus:latest --config.file=/etc/prometheus/prometheus.yml

# Grafana
container run -d --name apex-grafana \
    -p 3004:3000 \
    -e GF_SECURITY_ADMIN_USER=${GRAFANA_USER:-admin} \
    -e GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD:-admin} \
    -v "$(pwd)/infrastructure/monitoring/grafana/provisioning:/etc/grafana/provisioning" \
    -v "$(pwd)/infrastructure/docker/volumes/grafana:/var/lib/grafana" \
    index.docker.io/grafana/grafana:latest

# Jaeger
container run -d --name apex-jaeger \
    -p 16686:16686 -p 4317:4317 -p 4318:4318 \
    -e COLLECTOR_OTLP_ENABLED=true \
    index.docker.io/jaegertracing/all-in-one:latest

echo "✅ APEX V3 Successfully Started using Apple Containerization!"
echo "   Dashboard: http://localhost:3001"
echo "   Grafana:   http://localhost:3004"
echo "   Jaeger:    http://localhost:16686"
