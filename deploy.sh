#!/bin/bash
# ============================================
# APEX V3 — Remote Deploy Script
# Sources GCP config from .env at project root
# ============================================

set -euo pipefail

# Load GCP config from .env
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/.env" ]; then
  # shellcheck disable=SC1091
  set -a && source "$SCRIPT_DIR/.env" && set +a
else
  echo "ERROR: .env not found at $SCRIPT_DIR/.env" >&2
  exit 1
fi

# Validate required vars
: "${GCP_PROJECT_ID:?GCP_PROJECT_ID is not set in .env}"
: "${GCP_ZONE:?GCP_ZONE is not set in .env}"
: "${GCP_INSTANCE_NAME:?GCP_INSTANCE_NAME is not set in .env}"
: "${GCP_INSTANCE_USER:?GCP_INSTANCE_USER is not set in .env}"
: "${GCP_REMOTE_PATH:?GCP_REMOTE_PATH is not set in .env}"

GCP_ARGS="--zone \"$GCP_ZONE\" --project \"$GCP_PROJECT_ID\""

echo "==> Deploying to GCP instance: $GCP_INSTANCE_NAME ($GCP_ZONE)"
echo "==> Remote path: $GCP_REMOTE_PATH"
echo ""

# --- Upload files via SCP ---
echo "==> Uploading files..."

gcloud compute scp --zone "$GCP_ZONE" --project "$GCP_PROJECT_ID" \
  docker-compose.remote.yml \
  "$GCP_INSTANCE_NAME":~/docker-compose.yml

gcloud compute scp --zone "$GCP_ZONE" --project "$GCP_PROJECT_ID" \
  infrastructure/docker/Dockerfile.typescript \
  "$GCP_INSTANCE_NAME":~/Dockerfile.typescript

gcloud compute scp --zone "$GCP_ZONE" --project "$GCP_PROJECT_ID" \
  apps/api/src/index.ts \
  "$GCP_INSTANCE_NAME":~/index.ts

gcloud compute scp --zone "$GCP_ZONE" --project "$GCP_PROJECT_ID" \
  infrastructure/monitoring/loki/loki-config.yml \
  "$GCP_INSTANCE_NAME":~/loki-config.yml

gcloud compute scp --zone "$GCP_ZONE" --project "$GCP_PROJECT_ID" \
  infrastructure/monitoring/promtail/promtail-config.yml \
  "$GCP_INSTANCE_NAME":~/promtail-config.yml

echo "==> Files uploaded. Running remote commands..."

# --- Remote commands ---
gcloud compute ssh --zone "$GCP_ZONE" "$GCP_INSTANCE_NAME" \
  --project "$GCP_PROJECT_ID" --command "
    sudo mkdir -p ${GCP_REMOTE_PATH}/infrastructure/monitoring/loki
    sudo mkdir -p ${GCP_REMOTE_PATH}/infrastructure/monitoring/promtail
    sudo cp ~/docker-compose.yml     ${GCP_REMOTE_PATH}/infrastructure/docker/docker-compose.yml
    sudo cp ~/Dockerfile.typescript  ${GCP_REMOTE_PATH}/infrastructure/docker/Dockerfile.typescript
    sudo cp ~/index.ts               ${GCP_REMOTE_PATH}/apps/api/src/index.ts
    sudo cp ~/loki-config.yml        ${GCP_REMOTE_PATH}/infrastructure/monitoring/loki/loki-config.yml
    sudo cp ~/promtail-config.yml    ${GCP_REMOTE_PATH}/infrastructure/monitoring/promtail/promtail-config.yml
    sudo chown -R ${GCP_INSTANCE_USER}:${GCP_INSTANCE_USER} ${GCP_REMOTE_PATH}/infrastructure/monitoring
    sudo sh -c 'cd ${GCP_REMOTE_PATH}/infrastructure/docker && \
      docker compose build api dashboard orchestrator ai-council && \
      docker compose up -d'
"

echo ""
echo "✅ Deploy complete!"
