#!/bin/bash
# ============================================
# APEX V3 — Sequential Build & Deploy Script
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
: "${GCP_REMOTE_PATH:?GCP_REMOTE_PATH is not set in .env}"

echo "==> Sequential build & deploy on: $GCP_INSTANCE_NAME ($GCP_ZONE)"

gcloud compute ssh --zone "$GCP_ZONE" "$GCP_INSTANCE_NAME" \
  --project "$GCP_PROJECT_ID" --command "
    sudo sh -c 'cd ${GCP_REMOTE_PATH}/infrastructure/docker && \
      docker compose build api && \
      docker compose build dashboard && \
      docker compose build orchestrator && \
      docker compose build ai-council && \
      docker compose up -d'
"

echo ""
echo "✅ Sequential deploy complete!"
