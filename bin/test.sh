#!/usr/bin/env bash

# Systematic test runner for messaging service endpoints
# - Defines an array of test cases with method, path, headers, body, and expected status
# - Executes each test, verifies the HTTP status, and prints a summary
# - Compatible with macOS bash 3.x (no associative arrays used)

BASE_URL="${BASE_URL:-http://localhost:8080}"
# Default to showing response bodies; override with SHOW_BODY=false
SHOW_BODY="${SHOW_BODY:-true}"

# Resolve script directory for locating default tests file
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTS_FILE_DEFAULT="$SCRIPT_DIR/../tests/http/tests.json"
TESTS_FILE="${TESTS_FILE:-$TESTS_FILE_DEFAULT}"

echo "=== Messaging Service Test Runner ==="
echo "Base URL: $BASE_URL"
echo

# If modern tooling (jq) is available, prefer JSON-defined test cases for easier editing.
USE_JSON_TESTS=false
if command -v jq >/dev/null 2>&1 && [ -f "$TESTS_FILE" ]; then
  USE_JSON_TESTS=true
  echo "Using tests file: $TESTS_FILE"
fi

# Utility: join multiple -H flags from a single header string using '||' as separator
#   Example: "Header1: v1||Header2: v2"
build_header_args() {
  local header_str="$1"
  HEADER_ARGS=()
  if [ -n "$header_str" ]; then
    IFS='||' read -r -a parts <<< "$header_str"
    for p in "${parts[@]}"; do
      # trim leading/trailing spaces
      local t="$p"
      t="${t## }"; t="${t%% }"
      [ -n "$t" ] && HEADER_ARGS+=( -H "$t" )
    done
  fi
}

run_test() {
  local idx="$1" name="$2" method="$3" path="$4" headers="$5" body="$6" expect="$7" assert_filter="$8"

  build_header_args "$headers"

  # Auto-add JSON Content-Type if a body is present and no explicit Content-Type provided
  local have_ct=false
  for h in "${HEADER_ARGS[@]}"; do
    if [[ "$h" =~ Content-Type: ]]; then have_ct=true; break; fi
  done
  if [ -n "$body" ] && [ "$have_ct" = false ]; then
    HEADER_ARGS+=( -H "Content-Type: application/json" )
  fi

  local url="$BASE_URL$path"
  local tmp_body
  tmp_body=$(mktemp 2>/dev/null || echo "/tmp/messaging_test_body_$$")

  local code
  if [ -n "$body" ]; then
    code=$(curl -sS -o "$tmp_body" -w "%{http_code}" -X "$method" "$url" \
      "${HEADER_ARGS[@]}" \
      -d "$body")
  else
    code=$(curl -sS -o "$tmp_body" -w "%{http_code}" -X "$method" "$url" \
      "${HEADER_ARGS[@]}")
  fi

  local pass="FAIL"
  if [ "$code" = "$expect" ]; then
    pass="PASS"
  fi

  echo "[$idx] $name"
  echo "  $method $path"
  [ -n "$headers" ] && echo "  Headers: $headers"
  if [ -n "$body" ]; then
    echo "  Body JSON:"
    if command -v jq >/dev/null 2>&1; then
      echo "$body" | jq . 2>/dev/null | sed 's/^/    /'
    else
      echo "$body" | sed 's/^/    /'
    fi
  fi
  echo "  Expect: $expect  Got: $code  => $pass"
  if [ -n "$assert_filter" ]; then
    echo "  Assert (jq): $assert_filter"
  fi
  if [ "$SHOW_BODY" = "true" ]; then
    echo "  Response body:"
    if command -v jq >/dev/null 2>&1; then
      # Try to pretty print JSON responses; fall back to raw if not JSON
      if jq . >/dev/null 2>&1 < "$tmp_body"; then
        jq . < "$tmp_body" | sed 's/^/    /'
      else
        sed 's/^/    /' "$tmp_body"
      fi
    else
      sed 's/^/    /' "$tmp_body"
    fi
  fi
  echo

  # If we passed status code check and an assert is provided, evaluate jq assert against JSON body
  if [ "$pass" = "PASS" ] && [ -n "$assert_filter" ]; then
    if command -v jq >/dev/null 2>&1; then
      if jq -e . >/dev/null 2>&1 < "$tmp_body"; then
        if jq -e "$assert_filter" "$tmp_body" >/dev/null 2>&1; then
          echo "  Assert result: PASS"
        else
          echo "  Assert result: FAIL"
          pass="FAIL"
        fi
      else
        echo "  Assert skipped (response not JSON)"
      fi
    else
      echo "  Assert skipped (jq not found)"
    fi
  fi

  rm -f "$tmp_body" 2>/dev/null || true

  if [ "$pass" = "PASS" ]; then
    return 0
  else
    return 1
  fi
}

PASS_COUNT=0
FAIL_COUNT=0

if [ "$USE_JSON_TESTS" = true ]; then
  # Validate tests file schema before running
  validate_tests_schema() {
    local errs
    errs=$(jq -r '
      def is_int: (type=="number" and .==floor);
      def err($i;$f;$m): "Test[\($i)]: invalid \($f): \($m)";
      if type!="array" then "Top-level JSON must be an array" else empty end,
      to_entries[] as $e |
      # name
      ( $e.value.name | type ) as $t_name |
      if $t_name != "string" then err($e.key;"name";"expected string") else empty end,
      # method
      ( $e.value.method // "" ) as $m |
      if ($m|type)!="string" or (["GET","POST","PUT","PATCH","DELETE"] | index($m)) == null
      then err($e.key;"method";"must be one of GET,POST,PUT,PATCH,DELETE") else empty end,
      # path
      ( $e.value.path // "" ) as $p |
      if ($p|type)!="string" or ( $p | startswith("/") | not )
      then err($e.key;"path";"must be a string starting with '/'") else empty end,
      # headers
      ( $e.value.headers // [] ) as $h |
      if ($h|type)!="array" or ( [ $h[]? | type == "string" ] | all(.==true) | not )
      then err($e.key;"headers";"must be an array of strings") else empty end,
      # body
      ( $e.value.body ) as $b |
      if ($b!=null and ($b|type)!="object") then err($e.key;"body";"must be object or null") else empty end,
      # expect
      ( $e.value.expect ) as $x |
      if ($x|type)!="number" or ($x|floor)!=$x or ($x<100 or $x>599)
      then err($e.key;"expect";"must be integer HTTP code 100-599") else empty end,
      # assert (optional jq filter string)
      ( $e.value.assert ) as $a |
      if ($a!=null and ($a|type)!="string") then err($e.key;"assert";"must be string if present") else empty end
    ' "$TESTS_FILE")
    if [ -n "$errs" ]; then
      echo "Test schema validation failed for $TESTS_FILE:" >&2
      echo "$errs" >&2
      exit 2
    fi
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
  assert_filter=$(jq -r '(.assert // "")' <<< "$test")
    # jq -c returns '""' for empty string; strip surrounding quotes to get empty
    if [ "$body_json" = '""' ]; then body_json=""; fi

    if run_test "$INDEX" "$name" "$method" "$path" "$headers_joined" "$body_json" "$expect" "$assert_filter"; then
      PASS_COUNT=$((PASS_COUNT+1))
    else
      FAIL_COUNT=$((FAIL_COUNT+1))
    fi
    INDEX=$((INDEX+1))
  done < <(jq -c '.[]' "$TESTS_FILE")
else
  # Fallback: legacy arrays for environments without jq
  NAMES=(
    "Send SMS" "Send MMS" "Send Email" "Webhook SMS" "Webhook MMS" "Webhook Email"
    "List conversations" "List messages for conversation"
    "Idempotent SMS (first)" "Idempotent SMS (second)"
    "Idempotent Email (first)" "Idempotent Email (second)"
    "Idempotent Webhook SMS (first)" "Idempotent Webhook SMS (second)"
    "Invalid SMS type" "MMS without attachments" "Email empty body"
    "GET conversations unacceptable Accept" "MMS with empty attachments array"
  )
  METHODS=(POST POST POST POST POST POST GET GET POST POST POST POST POST POST POST POST POST GET POST)
  PATHS=(
    "/api/messages/sms" "/api/messages/sms" "/api/messages/email" "/api/webhooks/sms" "/api/webhooks/sms" "/api/webhooks/email"
    "/api/conversations" "/api/conversations/1/messages"
    "/api/messages/sms" "/api/messages/sms" "/api/messages/email" "/api/messages/email"
    "/api/webhooks/sms" "/api/webhooks/sms" "/api/messages/sms" "/api/messages/sms" "/api/messages/email" "/api/conversations" "/api/messages/sms"
  )
  HEADERS=(
    "" "" "" "" "" "" "" ""
    "Idempotency-Key: idem-sms-001" "Idempotency-Key: idem-sms-001"
    "Idempotency-Key: idem-email-001" "Idempotency-Key: idem-email-001"
    "Idempotency-Key: idem-wh-sms-001" "Idempotency-Key: idem-wh-sms-001"
    "" "" "" "Accept: text/plain" ""
  )
  BODIES=(
    '{"from":"+12016661234","to":"+18045551234","type":"sms","body":"Hello! This is a test SMS message.","attachments":null,"timestamp":"2024-11-01T14:00:00Z"}'
    '{"from":"+12016661234","to":"+18045551234","type":"mms","body":"Hello! This is a test MMS message with attachment.","attachments":["https://example.com/image.jpg"],"timestamp":"2024-11-01T14:00:00Z"}'
    '{"from":"user@usehatchapp.com","to":"contact@gmail.com","body":"Hello! This is a test email message with <b>HTML</b> formatting.","attachments":["https://example.com/document.pdf"],"timestamp":"2024-11-01T14:00:00Z"}'
    '{"from":"+18045551234","to":"+12016661234","type":"sms","messaging_provider_id":"message-1","body":"This is an incoming SMS message","attachments":null,"timestamp":"2024-11-01T14:00:00Z"}'
    '{"from":"+18045551234","to":"+12016661234","type":"mms","messaging_provider_id":"message-2","body":"This is an incoming MMS message","attachments":["https://example.com/received-image.jpg"],"timestamp":"2024-11-01T14:00:00Z"}'
    '{"from":"contact@gmail.com","to":"user@usehatchapp.com","xillio_id":"message-3","body":"<html><body>This is an incoming email with <b>HTML</b> content</body></html>","attachments":["https://example.com/received-document.pdf"],"timestamp":"2024-11-01T14:00:00Z"}'
    "" ""
    '{"from":"+12016661234","to":"+18045551234","type":"sms","body":"Idempotent test SMS.","attachments":null,"timestamp":"2024-11-01T14:05:00Z"}'
    '{"from":"+12016661234","to":"+18045551234","type":"sms","body":"Idempotent test SMS.","attachments":null,"timestamp":"2024-11-01T14:05:00Z"}'
    '{"from":"user@usehatchapp.com","to":"contact@gmail.com","body":"Idempotent email message.","attachments":["https://example.com/doc.pdf"],"timestamp":"2024-11-01T14:06:00Z"}'
    '{"from":"user@usehatchapp.com","to":"contact@gmail.com","body":"Idempotent email message.","attachments":["https://example.com/doc.pdf"],"timestamp":"2024-11-01T14:06:00Z"}'
    '{"from":"+18045551234","to":"+12016661234","type":"sms","messaging_provider_id":"message-idem-1","body":"Incoming idempotent SMS","attachments":null,"timestamp":"2024-11-01T14:07:00Z"}'
    '{"from":"+18045551234","to":"+12016661234","type":"sms","messaging_provider_id":"message-idem-1","body":"Incoming idempotent SMS","attachments":null,"timestamp":"2024-11-01T14:07:00Z"}'
    '{"from":"+12016661234","to":"+18045551234","type":"smsx","body":"Invalid type should fail","attachments":null,"timestamp":"2024-11-01T14:10:00Z"}'
    '{"from":"+12016661234","to":"+18045551234","type":"mms","body":"MMS requires at least one attachment","attachments":null,"timestamp":"2024-11-01T14:11:00Z"}'
    '{"from":"user@usehatchapp.com","to":"contact@gmail.com","body":"","attachments":null,"timestamp":"2024-11-01T14:12:00Z"}'
    ""
    '{"from":"+12016661234","to":"+18045551234","type":"mms","body":"Empty attachments array should fail","attachments":[],"timestamp":"2024-11-01T14:13:00Z"}'
  )
  EXPECTS=(202 202 202 202 202 202 200 200 202 202 202 202 202 202 400 400 400 406 400)

  TOTAL=${#NAMES[@]}
  for ((i=0; i<TOTAL; i++)); do
    name="${NAMES[$i]}"; method="${METHODS[$i]}"; path="${PATHS[$i]}"
    headers="${HEADERS[$i]}"; body="${BODIES[$i]}"; expect="${EXPECTS[$i]}"
    if run_test "$(($i+1))" "$name" "$method" "$path" "$headers" "$body" "$expect" ""; then
      PASS_COUNT=$((PASS_COUNT+1))
    else
      FAIL_COUNT=$((FAIL_COUNT+1))
    fi
  done
fi

echo "=== Summary ==="
echo "  Total: ${TOTAL:-$((PASS_COUNT+FAIL_COUNT))}  Passed: $PASS_COUNT  Failed: $FAIL_COUNT"

if [ "$FAIL_COUNT" -gt 0 ]; then
  exit 1
else
  exit 0
fi