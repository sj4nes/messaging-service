<!--
Sync Impact Report
- Version: N/A → 1.0.0
- Modified principles:
	- [PRINCIPLE_1_NAME] → Security-First and Secrets Hygiene (Non-Negotiable)
	- [PRINCIPLE_2_NAME] → Test-First and Quality Gates
	- [PRINCIPLE_3_NAME] → Observability and Auditability
	- [PRINCIPLE_4_NAME] → Versioning, Change Control, and Backwards Compatibility
	- [PRINCIPLE_5_NAME] → Simplicity and Single Responsibility
- Added sections: Named Section 2 (Safety, Security, and Compliance Requirements); Named Section 3 (Development Workflow, Reviews, and Quality Gates)
- Removed sections: None
- Templates reviewed/updated:
	- .specify/templates/plan-template.md ✅ updated (removed broken commands path reference)
	- .specify/templates/spec-template.md ✅ reviewed (no changes)
	- .specify/templates/tasks-template.md ✅ reviewed (no changes)
	- .specify/templates/checklist-template.md ✅ reviewed (no changes)
- Deferred TODOs: None (initial ratification and amendment set to today)
-->

# Messaging Service Constitution

## Core Principles

### Security-First and Secrets Hygiene (Non-Negotiable)
- MUST validate and sanitize all external inputs before processing.
- MUST never hardcode secrets; use an approved secret manager. No secrets in env variables, logs, or errors; ensure redaction.
- MUST authenticate and authorize all external interactions; enforce least privilege.
- MUST encrypt data in transit (TLS 1.2+) and at rest (AES‑256) with managed keys.
- MUST enforce rate limiting and pass SAST/DAST security scans before merge.
Rationale: Establishes defense-in-depth and blocks common attack vectors.

### Test-First and Quality Gates
- MUST practice TDD for new functionality (red‑green‑refactor) with tests written first.
- MUST maintain ≥80% unit test coverage; add integration tests for critical workflows.
- MUST run SAST, DAST, dependency, and license scanning in CI; failures block merges.
- MUST include edge/negative tests and maintain a regression suite for fixed bugs.
Rationale: Prevents regressions and sustains predictable quality.

### Observability and Auditability
- MUST emit structured, contextual logs with sensitive fields redacted.
- MUST generate immutable audit trails for significant actions and decisions.
- MUST expose metrics and traces for core operations and latency/error budgets.
- MUST enable real-time monitoring and alerting for anomalies and policy violations.
Rationale: Enables debugging, compliance verification, and reliability.

### Versioning, Change Control, and Backwards Compatibility
- MUST use Semantic Versioning; document breaking changes and provide migration notes.
- MUST require code review (≥2 approvals for protected branches) before merge.
- MUST record ADRs for major decisions and their consequences.
- MUST default to deny for permissions; risky changes follow explicit approval workflow.
Rationale: Ensures safe, predictable evolution of the system.

### Simplicity and Single Responsibility
- MUST keep modules focused; target cyclomatic complexity ≤10 per function and minimal nesting.
- SHOULD avoid premature optimization (YAGNI) and reduce coupling between modules.
- MUST document non-obvious logic with clear docstrings/comments.
- MUST keep public interfaces minimal and stable.
Rationale: Improves maintainability, readability, and onboarding.

## Safety, Security, and Compliance Requirements

- Input validation and output encoding are mandatory for all external inputs/outputs.
- Secrets management: store in external secret managers; never commit or log secrets; enable retrieval with authenticated service identity and audit trails.
- Access control: authenticate and authorize all API endpoints and database access; apply least privilege and mTLS where applicable.
- Data protection: encrypt data in transit (TLS 1.2+) and at rest (AES‑256); rotate keys regularly.
- Logging and privacy: redact PII and credentials; provide structured logs without sensitive data; retain per policy and ensure secure deletion.
- Security testing: SAST and DAST must pass pre-merge; weekly vulnerability scanning; criticals fixed within 24h, highs within 7 days.
- Abuse prevention: enforce user/IP rate limits; implement graceful degradation under load; alert on sustained attack patterns.
- Compliance: maintain SBOM and verify license compatibility; block builds on license conflicts; maintain audit evidence for reviews.

## Development Workflow, Reviews, and Quality Gates

- Workflow: Use feature branches with PRs; link specs/tasks to changes; protect main branch.
- Reviews: Require at least 2 approvals for significant changes; reviewers check security, tests, documentation, and adherence to this constitution.
- CI Gates: Lint/type checks, unit/integration tests, SAST/DAST, dependency and license scans must PASS before merge.
- Versioning: Follow SemVer; record changes in CHANGELOG and ADRs for architectural decisions.
- Releases: Include migration steps for breaking changes; provide rollbacks and kill switches for risky deploys.
- Observability: Ensure logs, metrics, and traces for new features; add alerts for new critical paths.
- Documentation: Keep README, quickstarts, and contracts updated alongside code.

## Governance

- Authority: This constitution supersedes ad‑hoc practices. Exceptions require explicit approval and documented risk.
- Amendment Procedure: Propose changes via PR updating this file. Include a Sync Impact Report summarizing changes and affected templates. Require 2 approvals.
- Versioning Policy: Bump version per semantic rules—MAJOR for incompatible removals/redefinitions, MINOR for new sections/principles or material expansions, PATCH for clarifications/typos. Set Last Amended to the merge date; Ratified is the original adoption date.
- Compliance Reviews: Conduct quarterly reviews; CI gates enforce day‑to‑day compliance. Violations block merges until resolved or exception approved.
- Traceability: Ensure all changes are attributable to specific actors with immutable history.

**Version**: 1.0.0 | **Ratified**: 2025-11-05 | **Last Amended**: 2025-11-05