#!/usr/bin/env bash

set -euo pipefail

# Simple launcher for the messaging service.
# Responsibilities:
# - Optionally start the database (docker-compose) unless START_DB=false
# - Optionally apply migrations if DATABASE_URL is set and APPLY_MIGRATIONS=true
# - Start the Rust server (messaging-server) on PORT (default 8080)
# - If a process is already listening on PORT, do not start another; override with FORCE_RESTART=true to restart it

ENVIRONMENT="${ENV:-development}"
PORT="${PORT:-8080}"
START_DB="${START_DB:-true}"
APPLY_MIGRATIONS="${APPLY_MIGRATIONS:-true}"
FORCE_RESTART="${FORCE_RESTART:-false}"

echo "Starting messaging-service"
echo "Environment: $ENVIRONMENT"
echo "Port: $PORT"

is_listening() {
	local port="$1"
	if command -v lsof >/dev/null 2>&1; then
		lsof -i TCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
		return $?
	elif command -v nc >/dev/null 2>&1; then
		nc -z 127.0.0.1 "$port" >/dev/null 2>&1
		return $?
	else
		# Fallback: try HTTP ping
		curl -sS --max-time 1 "http://127.0.0.1:$port/healthz" >/dev/null 2>&1
		return $?
	fi
}

kill_port_listeners() {
	local port="$1"
	if ! command -v lsof >/dev/null 2>&1; then
		echo "Cannot force restart: 'lsof' not available to discover PIDs on port $port" >&2
		return 1
	fi
	local pids
	# -t prints PIDs only; filter for LISTEN state
	pids=$(lsof -t -i TCP:"$port" -sTCP:LISTEN || true)
	if [ -z "$pids" ]; then
		echo "No listeners found on port $port"
		return 0
	fi
	echo "Terminating listeners on port $port: $pids"
	# Try graceful shutdown first
	kill $pids >/dev/null 2>&1 || true
	sleep 0.5
	# Force kill any remaining
	local still
	still=$(echo "$pids" | xargs -I{} sh -c 'ps -p {} -o pid= 2>/dev/null' | xargs echo || true)
	if [ -n "$still" ]; then
		echo "Forcing kill for: $still"
		kill -9 $still >/dev/null 2>&1 || true
	fi
}

wait_for_port_close() {
	local port="$1" timeout_ms="$2" interval_ms="$3"
	local waited=0
	while [ "$waited" -lt "$timeout_ms" ]; do
		if ! is_listening "$port"; then return 0; fi
		local sleep_s
		sleep_s=$(awk -v ms="$interval_ms" 'BEGIN { printf "%.3f", ms/1000 }')
		sleep "$sleep_s"
		waited=$((waited+interval_ms))
	done
	return 1
}

if is_listening "$PORT"; then
	if [ "$FORCE_RESTART" = "true" ]; then
		echo "Port $PORT is in use; attempting FORCE_RESTART..."
		kill_port_listeners "$PORT" || {
			echo "Failed to terminate existing listener(s) on $PORT" >&2
			exit 1
		}
		CLOSE_TIMEOUT_MS=${CLOSE_TIMEOUT_MS:-5000}
		CLOSE_POLL_MS=${CLOSE_POLL_MS:-100}
		if wait_for_port_close "$PORT" "$CLOSE_TIMEOUT_MS" "$CLOSE_POLL_MS"; then
			echo "Port $PORT freed. Proceeding to start server."
		else
			echo "Port $PORT still busy after $CLOSE_TIMEOUT_MS ms" >&2
			exit 1
		fi
	else
		echo "A service is already listening on port $PORT. Reusing it and exiting."
		exit 0
	fi
fi

if [ "$START_DB" = "true" ]; then
	if command -v docker-compose >/dev/null 2>&1; then
		echo "Starting database via docker-compose..."
		docker-compose up -d
	else
		echo "docker-compose not found; skipping DB start." >&2
	fi
fi

if [ "$APPLY_MIGRATIONS" = "true" ]; then
	if [ -n "${DATABASE_URL:-}" ]; then
		echo "Applying migrations with db-migrate..."
		cargo run -p db-migrate -- apply || {
			echo "Migrations failed. Continuing to start server anyway." >&2
		}
	else
		echo "DATABASE_URL not set; skipping migrations."
	fi
fi

echo "Running messaging-server..."
exec env PORT="$PORT" cargo run -p messaging-server