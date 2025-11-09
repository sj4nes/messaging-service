# Data Model: OWASP Security Hardening

Entities derived from spec and research. Technology-agnostic schema; examples align with Rust/SQLx but avoid implementation specifics.

## Entities

### SecurityVulnerability
- id: UUID
- cve: string | null
- cvss_score: number (0.0–10.0)
- owasp_category: enum [A01..A10]
- component: string (crate/module/file)
- severity: enum [critical, high, medium, low]
- discovered_at: timestamp
- status: enum [open, in_progress, mitigated, resolved]
- remediation: string (summary)

Validation:
- cvss_score in [0.0, 10.0]
- status transitions: open→in_progress→(mitigated|resolved); mitigated→resolved

### SecurityAuditLog
- id: UUID
- event_type: enum [auth_failure, auth_success, authorization_violation, rate_limit_trigger, key_rotation, config_change]
- user_id: string | null
- source_ip: string
- action: string
- outcome: enum [allowed, denied, error]
- timestamp: timestamp
- metadata: json (redacted)

Validation:
- redact PII/credentials before write
- timestamp monotonic within batch

### AuthenticationAttempt
- id: UUID
- user_id: string | null
- source_ip: string
- timestamp: timestamp
- result: enum [success, failure]
- failure_reason: string | null
- rl_bucket_state: json (window, count)

### DependencyManifest
- id: UUID
- package: string
- version: string
- cves: string[]
- severity_max: enum [critical, high, medium, low, none]
- scanned_at: timestamp
- approved: boolean

### KeyRotationEvent
- id: UUID
- key_id: string
- version_from: string
- version_to: string
- rotated_by: string (service identity)
- timestamp: timestamp
- notes: string

### RateLimitSnapshot
- id: UUID
- captured_at: timestamp
- per_ip_limit: integer
- per_sender_limit: integer
- counters: json (top N IPs/senders)

## Relationships
- AuthenticationAttempt emits SecurityAuditLog entries
- KeyRotationEvent emits SecurityAuditLog entries
- DependencyManifest items reference SecurityVulnerability rows (via cves) when applicable

## Derived Views
- security_posture_summary: counts by severity, category, and status
- auth_anomaly_view: bursty failures by IP/user over sliding windows

## Notes
- Implement append-only constraints for SecurityAuditLog via DB triggers or policies.
- Index event_type, timestamp, user_id for fast queries.
