#!/usr/bin/env bash
# deploy.sh — Manual deploy script for Ollama server
# Usage: ./deploy.sh [server_ip] [ssh_key_path]
#
# This script:
#   1. Builds the Docker image locally
#   2. Transfers it to the server via scp
#   3. Loads the image and starts the container on the server
#
# Prerequisites:
#   - Docker installed locally
#   - SSH access to the server (root@138.201.144.251)
#   - .env file configured with correct DATABASE_URL

set -euo pipefail

SERVER="${1:-138.201.144.251}"
SSH_USER="root"
SSH_KEY="${2:-~/.ssh/id_ed25519}"
PROJECT_DIR="/root/crm-quality-inspector"
IMAGE_NAME="crm-quality-inspector:latest"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== CRM Quality Inspector — Deploy Script ==="
echo "Server: ${SSH_USER}@${SERVER}"
echo "Project dir: ${PROJECT_DIR}"
echo ""

# Check prerequisites
command -v docker >/dev/null 2>&1 || { echo "Error: docker not found"; exit 1; }
command -v scp >/dev/null 2>&1 || { echo "Error: scp not found"; exit 1; }
command -v ssh >/dev/null 2>&1 || { echo "Error: ssh not found"; exit 1; }

# Verify .env exists
if [ ! -f .env ]; then
    echo "Warning: .env not found, copying from .env.example"
    cp .env.example .env
fi

# Check DATABASE_URL is configured
if grep -q "PG_USER_REDACTED\|ssdssd\|example" .env; then
    echo "Error: DATABASE_URL in .env contains placeholder values"
    echo "Please edit .env with the correct credentials"
    exit 1
fi

echo "Building Docker image..."
docker build -t "$IMAGE_NAME" .
echo "Image built: $(docker image inspect "$IMAGE_NAME" --format='{{.Size}}' | numfmt --to=iec)"

echo "Copying image to server..."
docker save "$IMAGE_NAME" | gzip | ssh -i "$SSH_KEY" "${SSH_USER}@${SERVER}" "docker load"
echo "Image transferred."

echo "Deploying on server..."
ssh -i "$SSH_KEY" "${SSH_USER}@${SERVER}" << EOF
set -e
cd ${PROJECT_DIR}

# Ensure .env is present
if [ ! -f .env ]; then
    cp .env.example .env
fi

# Ensure log volume exists
docker volume create crm_qi_logs 2>/dev/null || true

# Stop old containers gracefully
docker compose down --remove-orphans 2>/dev/null || true

# Start fresh
docker compose up -d --build

# Show status
echo ""
echo "=== Deployment Status ==="
docker compose ps
echo ""
echo "Log volume: \$(docker volume inspect crm_qi_logs --format '{{.Mountpoint}}')"
echo "App logs: /var/log/crm-qi/"
EOF

echo ""
echo "=== Deploy Complete ==="
echo "Application URL: http://${SERVER}:3000"
echo "Health check: curl http://${SERVER}:3000/api/health"
echo "Logs: ssh ${SSH_USER}@${SERVER} 'docker compose logs -f'"
echo "Log file: ssh ${SSH_USER}@${SERVER} 'tail -f /var/log/crm-qi/crm-quality-inspector.log'"
