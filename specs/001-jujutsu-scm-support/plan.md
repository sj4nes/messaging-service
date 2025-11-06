# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See project documentation for the execution workflow.

# Implementation Plan: Jujutsu SCM Support

**Branch**: `001-jujutsu-scm-support` | **Date**: 2025-11-05 | **Spec**: ../spec.md
**Input**: Feature specification from `/specs/001-jujutsu-scm-support/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See project documentation for the execution workflow.

## Summary
## Summary

Introduce first-class support for Jujutsu (JJ) as the primary VCS when a `.jj/`
directory exists. Prefer JJ over Git if both are present. Map “branch” semantics to
JJ bookmarks named `NNN-short-name` at the current revision `@`. Compute next feature
number using local JJ bookmarks and `specs/` directories. Preserve current Git-only
behavior. Ensure all automated commits use single-line messages compatible with
`jj commit -m`.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Bash (POSIX-compatible)  
**Primary Dependencies**: Jujutsu CLI (`jj`), Git CLI (`git`)  
**Storage**: N/A  
**Testing**: Bash test harness via scripts and dry-run commands; spot checks in a JJ-only, Git-only, and mixed repo  
**Target Platform**: macOS dev (zsh), Linux CI
**Project Type**: Single repository tooling (scripts under `.specify/scripts/bash/`)  
**Performance Goals**: Feature creation under 3 seconds in JJ-only repo  
**Constraints**: Single-line commit messages for JJ (`jj commit -m`), no remote JJ scanning  
**Scale/Scope**: Local repository workflows only (no server APIs)
## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Security-First: No secrets hardcoded; scripts redact and avoid logging sensitive data.
- Test-First & Quality Gates: Add script-level tests/dry-runs; CI runs lint/shellcheck (to be added in tasks).
- Observability: Scripts emit structured, concise logs; errors go to stderr.
- Versioning/Change Control: Changes documented; follow SemVer in script headers/CHANGELOG.
- Simplicity/SRP: Introduce a small VCS abstraction in `common.sh` to keep scripts focused.

Status: PASS (no violations expected at design time; tests and linting will be enforced in tasks.)
## Project Structure

### Documentation (this feature)

```text
specs/001-jujutsu-scm-support/
├── plan.md              # This file (/speckit.plan command output)
### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->
```text
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

```text
specs/001-jujutsu-scm-support/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

**Structure Decision**: Single repository scripts under `.specify/scripts/bash/` with
shared helpers in `common.sh`. All feature docs under `specs/001-jujutsu-scm-support/`.
## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| VCS abstraction shim | Unify JJ and Git handling | Duplicating logic across scripts increases risk |

## Phase 0: Outline & Research

Unknowns and decisions to research:
- JJ bookmark creation syntax and safest flags to target `@`.
- Listing bookmarks reliably for numbering.
- Best approach to detect `.jj/` vs `.git/` and prefer JJ when both exist.
- Mapping “branch” messaging to bookmarks in script outputs.

Research Tasks:
- Research JJ commands: `jj bookmark create`, `jj bookmark list`, `jj log` template keywords.
- Best practices for parsing JJ outputs in Bash (robust, portable, low-dependency).
- Safety guidance for single-line commit messaging with JJ.

Deliverable: `research.md` with decisions, rationale, and alternatives.

## Phase 1: Design & Contracts

Deliverables:
- `data-model.md`: Define VCS Context entity and interactions.
- `contracts/README.md`: No external API; internal script contracts only.
- `quickstart.md`: How to use spec-kit with JJ: feature creation, numbering, bookmarks.
- Agent context update: run `.specify/scripts/bash/update-agent-context.sh copilot` to add JJ context.

Re-check Constitution Gates after design: PASS. Observability/logging guidance and
security hygiene are embedded in scripts and docs.

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]  
**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]  
**Storage**: [if applicable, e.g., PostgreSQL, CoreData, files or N/A]  
**Testing**: [e.g., pytest, XCTest, cargo test or NEEDS CLARIFICATION]  
**Target Platform**: [e.g., Linux server, iOS 15+, WASM or NEEDS CLARIFICATION]
**Project Type**: [single/web/mobile - determines source structure]  
**Performance Goals**: [domain-specific, e.g., 1000 req/s, 10k lines/sec, 60 fps or NEEDS CLARIFICATION]  
**Constraints**: [domain-specific, e.g., <200ms p95, <100MB memory, offline-capable or NEEDS CLARIFICATION]  
**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
