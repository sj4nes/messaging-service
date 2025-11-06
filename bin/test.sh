#!/usr/bin/env bash

# Systematic test runner for messaging service endpoints
# - Defines an array of test cases with method, path, headers, body, and expected status
# - Executes each test, verifies the HTTP status, and prints a summary
# - Compatible with macOS bash 3.x (no associative arrays used)

BASE_URL="${BASE_URL:-http://localhost:8080}"
SHOW_BODY="${SHOW_BODY:-false}" # set SHOW_BODY=true to print response bodies

echo "=== Messaging Service Test Runner ==="
echo "Base URL: $BASE_URL"
echo

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
  local idx="$1" name="$2" method="$3" path="$4" headers="$5" body="$6" expect="$7"

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
  if [ "$SHOW_BODY" = "true" ]; then
    echo "  Response body:" && sed 's/^/    /' "$tmp_body"
  fi
  echo

  rm -f "$tmp_body" 2>/dev/null || true

  if [ "$pass" = "PASS" ]; then
    return 0
  else
    return 1
  fi
}

# Define test cases (parallel arrays)
NAMES=(
  "Send SMS"
  "Send MMS"
  "Send Email"
  "Webhook SMS"
  "Webhook MMS"
  "Webhook Email"
  "List conversations"
  "List messages for conversation"
  "Idempotent SMS (first)"
  "Idempotent SMS (second)"
  "Idempotent Email (first)"
  "Idempotent Email (second)"
  "Idempotent Webhook SMS (first)"
  "Idempotent Webhook SMS (second)"
  "Invalid SMS type"
  "MMS without attachments"
  "Email empty body"
  "GET conversations unacceptable Accept"
  "MMS with empty attachments array"
)

METHODS=(
  POST # 1 Send SMS
  POST # 2 Send MMS
  POST # 3 Send Email
  POST # 4 Webhook SMS
  POST # 5 Webhook MMS
  POST # 6 Webhook Email
  GET  # 7 List conversations
  GET  # 8 List messages for conversation
  POST # 9 Idempotent SMS (first)
  POST # 10 Idempotent SMS (second)
  POST # 11 Idempotent Email (first)
  POST # 12 Idempotent Email (second)
  POST # 13 Idempotent Webhook SMS (first)
  POST # 14 Idempotent Webhook SMS (second)
  POST # 15 Invalid SMS type
  POST # 16 MMS without attachments
  POST # 17 Email empty body
  GET  # 18 GET conversations unacceptable Accept
  POST # 19 MMS with empty attachments array
)

PATHS=(
  "/api/messages/sms"
  "/api/messages/sms"
  "/api/messages/email"
  "/api/webhooks/sms"
  "/api/webhooks/sms"
  "/api/webhooks/email"
  "/api/conversations"
  "/api/conversations/1/messages"
  "/api/messages/sms"
  "/api/messages/sms"
  "/api/messages/email"
  "/api/messages/email"
  "/api/webhooks/sms"
  "/api/webhooks/sms"
  "/api/messages/sms"
  "/api/messages/sms"
  "/api/messages/email"
  "/api/conversations"
  "/api/messages/sms"
)

# Headers per test (use '||' to separate multiple headers)
HEADERS=(
  ""
  ""
  ""
  ""
  ""
  ""
  ""
  ""
  "Idempotency-Key: idem-sms-001"
  "Idempotency-Key: idem-sms-001"
  "Idempotency-Key: idem-email-001"
  "Idempotency-Key: idem-email-001"
  "Idempotency-Key: idem-wh-sms-001"
  "Idempotency-Key: idem-wh-sms-001"
  ""
  ""
  ""
  "Accept: text/plain"
  ""
)

# Bodies (empty string means no -d will be sent)
BODIES=(
  '{
    "from": "+12016661234",
    "to": "+18045551234",
    "type": "sms",
    "body": "Hello! This is a test SMS message.",
    "attachments": null,
    "timestamp": "2024-11-01T14:00:00Z"
  }'
  '{
    "from": "+12016661234",
    "to": "+18045551234",
    "type": "mms",
    "body": "Hello! This is a test MMS message with attachment.",
    "attachments": ["https://example.com/image.jpg"],
    "timestamp": "2024-11-01T14:00:00Z"
  }'
  '{
    "from": "user@usehatchapp.com",
    "to": "contact@gmail.com",
    "body": "Hello! This is a test email message with <b>HTML</b> formatting.",
    "attachments": ["https://example.com/document.pdf"],
    "timestamp": "2024-11-01T14:00:00Z"
  }'
  '{
    "from": "+18045551234",
    "to": "+12016661234",
    "type": "sms",
    "messaging_provider_id": "message-1",
    "body": "This is an incoming SMS message",
    "attachments": null,
    "timestamp": "2024-11-01T14:00:00Z"
  }'
  '{
    "from": "+18045551234",
    "to": "+12016661234",
    "type": "mms",
    "messaging_provider_id": "message-2",
    "body": "This is an incoming MMS message",
    "attachments": ["https://example.com/received-image.jpg"],
    "timestamp": "2024-11-01T14:00:00Z"
  }'
  '{
    "from": "contact@gmail.com",
    "to": "user@usehatchapp.com",
    "xillio_id": "message-3",
    "body": "<html><body>This is an incoming email with <b>HTML</b> content</body></html>",
    "attachments": ["https://example.com/received-document.pdf"],
    "timestamp": "2024-11-01T14:00:00Z"
  }'
  ""
  ""
  '{
    "from": "+12016661234",
    "to": "+18045551234",
    "type": "sms",
    "body": "Idempotent test SMS.",
    "attachments": null,
    "timestamp": "2024-11-01T14:05:00Z"
  }'
  '{
    "from": "+12016661234",
    "to": "+18045551234",
    "type": "sms",
    "body": "Idempotent test SMS.",
    "attachments": null,
    "timestamp": "2024-11-01T14:05:00Z"
  }'
  '{
    "from": "user@usehatchapp.com",
    "to": "contact@gmail.com",
    "body": "Idempotent email message.",
    "attachments": ["https://example.com/doc.pdf"],
    "timestamp": "2024-11-01T14:06:00Z"
  }'
  '{
    "from": "user@usehatchapp.com",
    "to": "contact@gmail.com",
    "body": "Idempotent email message.",
    "attachments": ["https://example.com/doc.pdf"],
    "timestamp": "2024-11-01T14:06:00Z"
  }'
  '{
    "from": "+18045551234",
    "to": "+12016661234",
    "type": "sms",
    "messaging_provider_id": "message-idem-1",
    "body": "Incoming idempotent SMS",
    "attachments": null,
    "timestamp": "2024-11-01T14:07:00Z"
  }'
  '{
    "from": "+18045551234",
    "to": "+12016661234",
    "type": "sms",
    "messaging_provider_id": "message-idem-1",
    "body": "Incoming idempotent SMS",
    "attachments": null,
    "timestamp": "2024-11-01T14:07:00Z"
  }'
  '{
    "from": "+12016661234",
    "to": "+18045551234",
    "type": "smsx",
    "body": "Invalid type should fail",
    "attachments": null,
    "timestamp": "2024-11-01T14:10:00Z"
  }'
  '{
    "from": "+12016661234",
    "to": "+18045551234",
    "type": "mms",
    "body": "MMS requires at least one attachment",
    "attachments": null,
    "timestamp": "2024-11-01T14:11:00Z"
  }'
  '{
    "from": "user@usehatchapp.com",
    "to": "contact@gmail.com",
    "body": "",
    "attachments": null,
    "timestamp": "2024-11-01T14:12:00Z"
  }'
  ""
  '{
    "from": "+12016661234",
    "to": "+18045551234",
    "type": "mms",
    "body": "Empty attachments array should fail",
    "attachments": [],
    "timestamp": "2024-11-01T14:13:00Z"
  }'
)

EXPECTS=(
  202 202 202 202 202 202 200 200
  202 202 202 202 202 202 400 400 400 406 400
)

# Execute all tests
TOTAL=${#NAMES[@]}
PASS_COUNT=0
FAIL_COUNT=0

for ((i=0; i<TOTAL; i++)); do
  name="${NAMES[$i]}"
  method="${METHODS[$i]}"
  path="${PATHS[$i]}"
  headers="${HEADERS[$i]}"
  body="${BODIES[$i]}"
  expect="${EXPECTS[$i]}"

  if run_test "$((i+1))" "$name" "$method" "$path" "$headers" "$body" "$expect"; then
    PASS_COUNT=$((PASS_COUNT+1))
  else
    FAIL_COUNT=$((FAIL_COUNT+1))
  fi
done

echo "=== Summary ==="
echo "  Total: $TOTAL  Passed: $PASS_COUNT  Failed: $FAIL_COUNT"

if [ "$FAIL_COUNT" -gt 0 ]; then
  exit 1
else
  exit 0
fi