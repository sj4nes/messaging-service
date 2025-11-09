# Quickstart: OWASP Security Hardening

## Purpose
Guide implementers to enable, verify, and monitor security controls introduced by Feature 010.

## Prerequisites
- Rust stable (auto-managed via Docker) 
- Docker + docker-compose
- HashiCorp Vault (prod) or mock secret provider (dev)
- `.env` configured (non-sensitive toggles only)

## Steps

### 1. Build & Run
```bash
docker compose up --build -d
curl -s http://localhost:8080/healthz
```
Expect `{"status":"ok"}`.

### 2. Run Security Scans
```bash
# Dependency CVE scan
cargo audit
# License and advisory scan
cargo deny check advisories licenses
# Container image scan (requires Trivy installed)
trivy image messaging-service:latest
# DAST baseline scan
zap-baseline.py -t http://localhost:8080 -r zap-report.html
```

### 3. Verify Authentication & Authorization
```bash
# Attempt protected endpoint without token
curl -i http://localhost:8080/security/vulnerabilities
# Expect 401
```

### 4. Check Security Headers
```bash
curl -i http://localhost:8080/security/health | grep -E 'Content-Security-Policy|Strict-Transport-Security|X-Frame-Options'
```

### 5. Inspect Logs (Redaction)
Review running container logs; confirm no passwords, tokens, or PII appear.

### 6. Trigger Rate Limiting
```bash
for i in $(seq 1 130); do curl -s http://localhost:8080/auth/login -d '{"username":"u","password":"p"}' -H 'Content-Type: application/json' >/dev/null; done
```
Check metrics endpoint (planned) for incremented `rate_limited` counts.

### 7. Key Rotation (Future)
Placeholder endpoint: `POST /auth/rotate-keys` returns 202 when implemented.

### 8. Review Security Posture
```bash
curl -s http://localhost:8080/security/health | jq
```

## Success Criteria Validation Map
| Criterion | Verification Step |
|-----------|-------------------|
| SC-001 | cargo audit / Trivy reports show no critical/high CVEs |
| SC-002 | `openapi.yaml` + test suite coverage summary |
| SC-003 | Integration tests assert 401/403 on unauth access |
| SC-004 | DB inspection script verifies hashed/encrypted fields |
| SC-005 | grep + sqlx compile checks (no dynamic SQL) |
| SC-006 | Log inspection + audit log table row count |
| SC-007 | CI pipeline includes cargo audit/deny steps |
| SC-008 | Incident playbook timestamps (manual for now) |
| SC-009 | CI security test suite passes (exit code 0) |
| SC-010 | Monitoring dashboards show zero incidents over window |

## Troubleshooting
| Issue | Cause | Resolution |
|-------|-------|------------|
| Build fails on cargo audit | Vulnerable dependency | Update dependency or apply patch version |
| Headers missing | Middleware not registered | Ensure security layer included in Axum router setup |
| High false positives rate limit | Defaults too strict | Adjust env limit variables and redeploy |
| Secrets appear in logs | Redaction not applied | Audit tracing subscriber configuration |

## Next Steps
- Implement audit log storage & redaction layer
- Integrate Vault client and replace placeholder secret loading
- Add security integration tests for injection and SSRF
