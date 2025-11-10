#!/usr/bin/env bash

# Copy of test.sh tailored for the Rust messaging-server (listening on 8083 via docker-compose)
# Differences:
# - Default BASE_URL points to port 8083
# - Starts Rust server with 'make run-server' (PORT overridden to 8083)
# - Keeps identical test logic and schema validation

BASE_URL="${BASE_URL:-http://localhost:8083}"
SHOW_BODY="${SHOW_BODY:-true}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTS_FILE_DEFAULT="$SCRIPT_DIR/../tests/http/tests.json"
TESTS_FILE="${TESTS_FILE:-$TESTS_FILE_DEFAULT}"

echo "=== Rust Messaging Service Test Runner ==="
echo "Base URL: $BASE_URL"
echo

START_SERVER="${START_SERVER:-true}"
SERVER_LOG="${SERVER_LOG:-server-rust.log}"

BASE_SCHEME="${BASE_URL%%://*}"
_after_scheme="${BASE_URL#*://}"
_hostport="${_after_scheme%%/*}"
if printf '%s' "$_hostport" | grep -q ':'; then
  BASE_HOST="${_hostport%%:*}"
  BASE_PORT="${_hostport##*:}"
else
  BASE_HOST="$_hostport"
  BASE_PORT=""
fi
if [ -z "$BASE_PORT" ]; then
  if [ "$BASE_SCHEME" = "https" ]; then BASE_PORT=443; else BASE_PORT=80; fi
fi

REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
STARTED_SERVER="false"
SERVER_PID=""

is_listening() {
  if command -v lsof >/dev/null 2>&1; then
    lsof -i TCP:"$1" -sTCP:LISTEN >/dev/null 2>&1
    return $?
  elif command -v nc >/dev/null 2>&1; then
    nc -z "$BASE_HOST" "$1" >/dev/null 2>&1
    return $?
  else
    curl -sS --max-time 1 "$BASE_URL" >/dev/null 2>&1
    return $?
  fi
}

wait_for_port() {
  local port="$1" timeout_ms="$2" interval_ms="$3"
  local waited=0
  while [ "$waited" -lt "$timeout_ms" ]; do
    if is_listening "$port"; then return 0; fi
    local sleep_s
    sleep_s=$(awk -v ms="$interval_ms" 'BEGIN { printf "%.3f", ms/1000 }')
    sleep "$sleep_s"
    waited=$((waited+interval_ms))
  done
  return 1
}

wait_for_metrics_counter() {
  local field="$1" min="$2" timeout_ms="${3:-5000}" interval_ms="${4:-250}"
  if ! command -v jq >/dev/null 2>&1; then
    echo "jq not found; skipping metrics wait for '$field'" >&2
    return 0
  fi
  local waited=0 url="$BASE_URL/metrics" tmp
  tmp=$(mktemp 2>/dev/null || echo "/tmp/messaging_rust_metrics_$$")
  echo "Waiting for /metrics: .$field >= $min (timeout ${timeout_ms}ms)"
  while [ "$waited" -lt "$timeout_ms" ]; do
    local code
    code=$(curl -sS -H 'Accept: application/json' -o "$tmp" -w "%{http_code}" "$url" || echo 000)
    if [ "$code" = "200" ] && jq -e . >/dev/null 2>&1 < "$tmp"; then
      local val
      val=$(jq -r ".${field} // 0 | tonumber" "$tmp" 2>/dev/null || echo 0)
      if [ -n "$val" ] && [ "$val" -ge "$min" ]; then
        rm -f "$tmp" 2>/dev/null || true
        echo "Metrics condition met: $field=$val"
        return 0
      fi
    fi
    local sleep_s
    sleep_s=$(awk -v ms="$interval_ms" 'BEGIN { printf "%.3f", ms/1000 }')
    sleep "$sleep_s"
    waited=$((waited+interval_ms))
  done
  echo "Timed out waiting for metrics '$field' >= $min; continuing tests" >&2
  rm -f "$tmp" 2>/dev/null || true
  return 0
}

cleanup_server() {
  if [ "$STARTED_SERVER" = "true" ] && [ -n "$SERVER_PID" ]; then
    if ps -p "$SERVER_PID" >/dev/null 2>&1; then
      echo "Stopping background Rust server (pid=$SERVER_PID)"
      kill "$SERVER_PID" >/dev/null 2>&1 || true
      sleep 0.5
      if ps -p "$SERVER_PID" >/dev/null 2>&1; then kill -9 "$SERVER_PID" >/dev/null 2>&1 || true; fi
    fi
  fi
}
trap cleanup_server EXIT INT TERM

if [ "$START_SERVER" = "true" ]; then
  if is_listening "$BASE_PORT"; then
    echo "Rust server already listening on $BASE_HOST:$BASE_PORT — will not start a new one."
  else
    echo "Starting Rust server in background via 'make run-server' (log: $SERVER_LOG)"
    (
      cd "$REPO_ROOT" || exit 1
      PORT="$BASE_PORT" make run-server >"$SERVER_LOG" 2>&1
    ) &
    SERVER_PID=$!
    STARTED_SERVER="true"
    START_TIMEOUT_MS=${SERVER_START_TIMEOUT_MS:-20000}
    POLL_INTERVAL_MS=${SERVER_START_POLL_MS:-250}
    if wait_for_port "$BASE_PORT" "$START_TIMEOUT_MS" "$POLL_INTERVAL_MS"; then
      echo "Rust server is up on $BASE_HOST:$BASE_PORT"
    else
      echo "Rust server failed to start within $START_TIMEOUT_MS ms. Tail of log:" >&2
      tail -n 100 "$SERVER_LOG" >&2 || true
      exit 1
    fi
  fi
fi

USE_JSON_TESTS=false
if command -v jq >/dev/null 2>&1 && [ -f "$TESTS_FILE" ]; then
  USE_JSON_TESTS=true
  echo "Using tests file: $TESTS_FILE"
fi

build_header_args() {
  local header_str="$1"
  HEADER_ARGS=()
  if [ -n "$header_str" ]; then
    IFS='||' read -r -a parts <<< "$header_str"
    for p in "${parts[@]}"; do
      local t="$p"; t="${t## }"; t="${t%% }"
      [ -n "$t" ] && HEADER_ARGS+=( -H "$t" )
    done
  fi
}

run_test() {
  local idx="$1" name="$2" method="$3" path="$4" headers="$5" body="$6" expect="$7" assert_filter="$8" delay_ms="$9" assert_poll_ms="${10}" assert_max_tries="${11}"
  if [ -n "$delay_ms" ] && [ "$delay_ms" != "0" ]; then
    local delay_secs
    delay_secs=$(awk -v ms="$delay_ms" 'BEGIN { printf "%.3f", ms/1000 }')
    echo "  (delaying ${delay_ms}ms before request)"
    sleep "$delay_secs"
  fi
  build_header_args "$headers"
  local have_ct=false
  for h in "${HEADER_ARGS[@]}"; do
    if [[ "$h" =~ Content-Type: ]]; then have_ct=true; break; fi
  done
  if [ -n "$body" ] && [ "$have_ct" = false ]; then
    HEADER_ARGS+=( -H "Content-Type: application/json" )
  fi
  local url="$BASE_URL$path" tmp_body
  tmp_body=$(mktemp 2>/dev/null || echo "/tmp/messaging_rust_test_body_$$")
  local code
  if [ -n "$body" ]; then
    code=$(curl -sS -o "$tmp_body" -w "%{http_code}" -X "$method" "$url" "${HEADER_ARGS[@]}" -d "$body")
  else
    code=$(curl -sS -o "$tmp_body" -w "%{http_code}" -X "$method" "$url" "${HEADER_ARGS[@]}")
  fi
  local pass="FAIL"
  [ "$code" = "$expect" ] && pass="PASS"
  echo "[$idx] $name"; echo "  $method $path"; [ -n "$headers" ] && echo "  Headers: $headers"
  if [ -n "$body" ]; then
    echo "  Body JSON:"; if command -v jq >/dev/null 2>&1; then echo "$body" | jq . 2>/dev/null | sed 's/^/    /'; else echo "$body" | sed 's/^/    /'; fi
  fi
  echo "  Expect: $expect  Got: $code  => $pass"
  if [ "$SHOW_BODY" = "true" ]; then
    echo "  Response body:"; if command -v jq >/dev/null 2>&1; then if jq . >/dev/null 2>&1 < "$tmp_body"; then jq . < "$tmp_body" | sed 's/^/    /'; else sed 's/^/    /' "$tmp_body"; fi; else sed 's/^/    /' "$tmp_body"; fi
  fi
  echo
  rm -f "$tmp_body" 2>/dev/null || true
  [ "$pass" = "PASS" ]
}

PASS_COUNT=0
FAIL_COUNT=0

if [ "$USE_JSON_TESTS" = true ]; then
  validate_tests_schema() {
    local errs
    errs=$(jq -r '
      def is_int: (type=="number" and .==floor);
      def err($i;$f;$m): "Test[\($i)]: invalid \($f): \($m)";
      if type!="array" then "Top-level JSON must be an array" else empty end,
      to_entries[] as $e |
      ( $e.value.name | type ) as $t_name |
      if $t_name != "string" then err($e.key;"name";"expected string") else empty end,
      ( $e.value.method // "" ) as $m |
      if ($m|type)!="string" or (["GET","POST","PUT","PATCH","DELETE"] | index($m)) == null
      then err($e.key;"method";"must be one of GET,POST,PUT,PATCH,DELETE") else empty end,
      ( $e.value.path // "" ) as $p |
      if ($p|type)!="string" or ( $p | startswith("/") | not )
      then err($e.key;"path";"must start with '/'") else empty end,
      ( $e.value.headers // [] ) as $h |
      if ($h|type)!="array" or ( [ $h[]? | type == "string" ] | all(.==true) | not )
      then err($e.key;"headers";"must be array of strings") else empty end,
      ( $e.value.body ) as $b |
      if ($b!=null and ($b|type)!="object") then err($e.key;"body";"must be object or null") else empty end,
      ( $e.value.expect ) as $x |
      if ($x|type)!="number" or ($x|floor)!=$x or ($x<100 or $x>599)
      then err($e.key;"expect";"must be HTTP code") else empty end,
      ( $e.value.assert ) as $a |
      if ($a!=null and ($a|type)!="string") then err($e.key;"assert";"must be string if present") else empty end
    ' "$TESTS_FILE")
    if [ -n "$errs" ]; then echo "Test schema validation failed:" >&2; echo "$errs" >&2; exit 2; fi
  }
  validate_tests_schema
  TOTAL=$(jq 'length' "$TESTS_FILE")
  INDEX=1
  while IFS= read -r test; do
    name=$(jq -r '.name' <<< "$test")
    method=$(jq -r '.method' <<< "$test")
    path=$(jq -r '.path' <<< "$test")
    expect=$(jq -r '.expect' <<< "$test")
    headers_joined=$(jq -r '(.headers // []) | join("||")' <<< "$test")
    body_json=$(jq -c 'if .body == null then "" else .body end' <<< "$test")
    if [ "$body_json" = '""' ]; then body_json=""; fi
    if run_test "$INDEX" "$name" "$method" "$path" "$headers_joined" "$body_json" "$expect" "" "0" "0" "1"; then
      PASS_COUNT=$((PASS_COUNT+1))
    else
      FAIL_COUNT=$((FAIL_COUNT+1))
    fi
    INDEX=$((INDEX+1))
  done < <(jq -c '.[]' "$TESTS_FILE")
else
  # For Rust legacy fallback we can reuse simple arrays from original if needed; omitted for brevity.
  echo "JSON tests file required for rust test script; none found. Set TESTS_FILE or install jq." >&2
  exit 3
fi

echo "=== Summary (Rust) ==="
echo "  Total: ${TOTAL:-$((PASS_COUNT+FAIL_COUNT))}  Passed: $PASS_COUNT  Failed: $FAIL_COUNT"

if [ "$FAIL_COUNT" -gt 0 ]; then exit 1; else exit 0; fi
