#!/usr/bin/env bash
set -euo pipefail

# ZAP Baseline scan against a target URL
# Usage: bin/zap-baseline.sh http://localhost:8080
# Produces: zap-report.html in repo root (or CWD)

TARGET_URL="${1:-http://localhost:8080}"
REPORT_FILE="${2:-zap-report.html}"

# Prefer Docker image to avoid local Java/ZAP installs
# On macOS, localhost from container may need host.docker.internal
if [[ "$(uname -s)" == "Darwin" && "$TARGET_URL" == http://localhost:* ]]; then
  TARGET_URL="${TARGET_URL/localhost/host.docker.internal}"
fi

echo "[ZAP] Scanning target: $TARGET_URL"

docker run --rm \
  -u "$(id -u):$(id -g)" \
  -v "$(pwd)":/zap/wrk \
  -t owasp/zap2docker-stable \
  zap-baseline.py -t "$TARGET_URL" -r "$(basename "$REPORT_FILE")" -d || true

echo "[ZAP] Report written to $REPORT_FILE"
