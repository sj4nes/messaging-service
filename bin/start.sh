#!/usr/bin/env bash

set -euo pipefail

ENVIRONMENT="${ENV:-development}"
PORT="${PORT:-8080}"

echo "Starting messaging-service"
echo "Environment: $ENVIRONMENT"
echo "Port: $PORT"

# Check for docker-compose
if ! command -v docker-compose >/dev/null 2>&1; then
    echo "Error: docker-compose not found. Please install Docker Compose." >&2
    echo "See: https://docs.docker.com/compose/install/" >&2
    exit 1
fi

# Start all services via docker-compose
echo "Starting services via docker-compose..."
docker-compose up -d

# Wait for services to be healthy
echo "Waiting for services to be ready..."
sleep 3

# Check if the service is responding
if command -v curl >/dev/null 2>&1; then
    MAX_RETRIES=30
    RETRY_COUNT=0
    while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
        if curl -sS --max-time 2 "http://localhost:$PORT/healthz" >/dev/null 2>&1; then
            echo "Service is ready at http://localhost:$PORT"
            echo "Health check: http://localhost:$PORT/healthz"
            exit 0
        fi
        RETRY_COUNT=$((RETRY_COUNT + 1))
        sleep 1
    done
    echo "Warning: Service did not respond within expected time. Check logs with: docker-compose logs" >&2
else
    echo "Service started. Check status with: docker-compose ps"
    echo "View logs with: docker-compose logs -f"
fi

exit 0 