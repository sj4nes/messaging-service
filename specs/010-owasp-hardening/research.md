# Research: OWASP Top 20 Security Hardening

**Branch**: 010-owasp-hardening  
**Date**: 2025-11-08  
**Scope**: Resolve Phase 0 unknowns and establish decisions, rationale, and alternatives.

## Decisions

### 1) Password Hashing Algorithm
- Decision: Use Argon2id with parameters: m=64MB, t=3, p=1 (tunable by env)
- Rationale: Memory-hard, resistant to GPU attacks; widely recommended over bcrypt for new systems.
- Alternatives: bcrypt (mature, widely supported but not memory-hard), scrypt (also memory-hard but less common in Rust ecosystem).

### 2) Secrets Management / KMS
- Decision: For local/dev: .env only for non-sensitive toggles; no secrets checked-in. For production: integrate with HashiCorp Vault via sidecar or agent; store DB creds and encryption keys; app retrieves tokens via machine identity.
- Rationale: Vault provides audit trails, policy, and dynamic secrets; avoids plaintext env and source.
- Alternatives: AWS Secrets Manager + KMS (good if AWS locked-in), GCP Secret Manager + KMS.

### 3) TLS Termination
- Decision: TLS terminated at ingress/proxy (e.g., Nginx/Traefik) with mTLS support optional; service communicates over localhost/overlay network with mTLS optional future.
- Rationale: Separation of concerns; easier cert rotation; consistent policy across services.
- Alternatives: In-service TLS via Rustls (more complexity per service).

### 4) Data-at-Rest Encryption
- Decision: Application-level column encryption for highly sensitive fields using AES-256-GCM; keys from Vault; key ID stored alongside ciphertext; rotate via re-encryption job.
- Rationale: Protects against DB snapshot exfiltration; crypto agility.
- Alternatives: Database-level TDE only (insufficient alone), pgcrypto (simple but key mgmt harder).

### 5) Rate Limit Sizing
- Decision: Start with defaults documented (per-IP 120/min, per-sender 60/min) and make configurable via env; add burst tokens with small bucket.
- Rationale: Conservative defaults; tune with production metrics.
- Alternatives: Static strict limits (risk false positives), dynamic adaptive algorithms (complexity).

### 6) Outbound Allowlist for SSRF
- Decision: Static allowlist via config for outbound calls; deny RFC1918, link-local, metadata endpoints by default.
- Rationale: Simplicity and safety; extend to dynamic list later.
- Alternatives: DNS pinning, egress proxy enforcement.

### 7) DAST Tooling
- Decision: OWASP ZAP baseline scan in CI against locally started service (docker-compose profile); fail build on high/critical findings.
- Rationale: Standard tool; easy to automate.
- Alternatives: Nikto (limited), custom curl fuzzers.

### 8) MFA Scope
- Decision: Defer MFA to a separate feature unless required by stakeholders; include re-auth for sensitive ops now.
- Rationale: Keep current scope manageable while tackling highest risks.
- Alternatives: Implement TOTP/WebAuthn now (larger scope).

### 9) Key Rotation Cadence
- Decision: Rotate encryption keys quarterly or on incident; implement key versioning and re-encrypt job.
- Rationale: Balance security with operational cost.
- Alternatives: Monthly (higher cost), yearly (higher risk).

### 10) Audit Log Storage
- Decision: Store security audit logs in PostgreSQL table with append-only constraints (trigger-based guard) and ship to external aggregator (optional) with redaction.
- Rationale: Queryable for forensics; tamper-evident via DB constraints.
- Alternatives: Files (harder to query), external-only (vendor lock-in).

## Best Practices Notes
- Argon2 parameters should be revisited per hardware; target ~100ms hash time in production.
- Use SQLx compile-time checks with `.sqlx/` cache; grep to forbid string-concatenated SQL.
- Trivy scan both base images and built images; update base image regularly.
- Use `tracing` fields redaction; avoid logging tokens/PII.
- Implement centralized error handling returning generic messages; detailed in logs only.

## Open Questions (Resolved)
- Identity provider integration: Out-of-scope; local auth only for now with pluggable design for future OIDC.
- Secrets bootstrapping in dev: Use `.env` with non-sensitive toggles; mock Vault in local if needed; document.

## Outcome
All Phase 0 unknowns are resolved with concrete decisions and documented alternatives. Proceeding to Phase 1 design & contracts.
