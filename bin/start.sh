#!/usr/bin/env bash

set -euo pipefail

ENVIRONMENT="${ENV:-development}"
PORT="${PORT:-8080}"

echo "Starting messaging-service"
echo "Environment: $ENVIRONMENT"
echo "Port: $PORT"

# Check for make
if ! command -v make >/dev/null 2>&1; then
	echo "Error: make not found. Please install make." >&2
	exit 1
fi

# Build Docker images, start database, and seed data
echo "Building Docker images..."
make docker-build

echo "Starting database..."
make db-up

echo "Seeding database..."
make db-seed

# Wait for services to be ready
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
