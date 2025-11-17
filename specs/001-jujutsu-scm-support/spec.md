# Feature Specification: Jujutsu SCM Support

**Feature Branch**: `001-jujutsu-scm-support`  
**Created**: 2025-11-05  
**Status**: Draft  
**Input**: User description: "support-jujutsu-scm There are a lot of scripts in spec-key that assume that we use GIT as the SCM. We actually use Jujutsu because it dramatically simplifies commits.  The single most important concept to realize is jujutsu doesn't have branches, but bookmarks. You do not checkout a branch to create it in jujutsu, you have to create a bookmark with the current revision (symbolized by @).  In the jujutsu log so long as a version has the bookmark of a feature in its latest history that can be considered the \"branch.\"  Unlike GIT jj commit doesn't support multiline messages from the command line so you should only focus on creating the most useful message in this limited space.  Some jj help is attached: #file:jujutsu.md   The changes are to check for the existence of a .jj folder and if seen takes priority over the usage of GIT."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Create Feature in JJ Repo (Priority: P1)

A developer runs the feature creation command in a repository that uses Jujutsu. The
system must prefer JJ over Git, compute the next feature number, create a JJ bookmark
as the “branch,” and scaffold the spec directory without invoking Git.

**Why this priority**: Enables core workflow in JJ environments; unblocks all other
spec-kit commands in JJ-first repos.

**Independent Test**: Run the feature creation script in a JJ-only repo (with `.jj/`),
verify a bookmark `NNN-short-name` is created at `@`, and validate that a spec
directory is scaffolded under `specs/NNN-short-name/` with no Git commands executed.

**Acceptance Scenarios**:

1. **Given** a repo with `.jj/` and no `.git/`, **When** the developer runs the
  feature creation command, **Then** a bookmark `001-jujutsu-scm-support` is created
  pointing to `@` and `specs/001-jujutsu-scm-support/spec.md` is generated.
2. **Given** a repo with both `.jj/` and `.git/`, **When** the developer runs the
  command, **Then** JJ is prioritized (no Git branch is created) and the JJ
  bookmark is created successfully.

---

### User Story 2 - Numbering and Collision Handling (Priority: P2)

A developer creates multiple features with the same short-name over time. The system
must detect the highest existing number across JJ bookmarks and `specs/` directories
and assign the next sequential number.

**Why this priority**: Prevents collisions and ensures traceability across features.

**Independent Test**: Create two features with the same short-name. Verify the second
feature uses `NNN+1` and does not overwrite existing bookmarks or directories.

**Acceptance Scenarios**:

1. **Given** an existing bookmark `001-jujutsu-scm-support` and matching spec dir,
   **When** a new feature is created with the same short-name, **Then** the new
   bookmark and directory are `002-jujutsu-scm-support`.

---

### User Story 3 - Git Fallback Compatibility (Priority: P3)

In a Git-only repository (no `.jj/`), current behavior remains unchanged. The
workflow continues to create a Git branch and scaffold the spec directory.

**Why this priority**: Preserves backwards compatibility for Git-only users.

**Independent Test**: Run the feature creation command in a Git-only repo and
confirm that a Git branch is created and `specs/` directory is scaffolded.

**Acceptance Scenarios**:

1. **Given** a repo with `.git/` and no `.jj/`, **When** the command runs, **Then**
  a Git branch `NNN-short-name` is created and `specs/NNN-short-name/` exists.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

- No `.git/` and no `.jj/`: Fail with a clear error explaining required repository
  initialization and how to proceed with Jujutsu or Git.
- Bookmark and spec mismatch: If `specs/` directory exists without a matching
  bookmark (or vice versa), the highest numeric prefix across both sources governs
  the next number.
- Remote state: Numbering uses local bookmarks and `specs/` directories only; remotes
  are ignored by default to avoid JJ/Git bridging assumptions.
- Commit message constraints: All automated commits must supply a single-line
  message suitable for `jj commit -m`.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: Detect `.jj/` presence and set VCS context to Jujutsu. If both `.jj/`
  and `.git/` are present, Jujutsu MUST be preferred for all operations.
- **FR-002**: Compute next feature number by scanning JJ bookmarks and
  `specs/NNN-short-name/` directories for the exact short-name; pick `max(N)+1`.
- **FR-003**: In JJ mode, create a bookmark at `@` named `NNN-short-name`; do not
  create a Git branch.
- **FR-004**: In JJ mode, avoid invoking any Git commands (e.g., `git fetch`,
  `git checkout`).
- **FR-005**: All automated commits performed by scripts MUST use a single-line
  message passed via `-m` (no multi-line content).
- **FR-006**: In Git-only repositories, preserve current behavior (create a branch
  `NNN-short-name` and scaffold spec directory).
- **FR-007**: Update impacted scripts to use a VCS abstraction that supports JJ and
  Git: `.specify/scripts/bash/{create-new-feature.sh,check-prerequisites.sh,common.sh,setup-plan.sh,update-agent-context.sh}`.
- **FR-008**: Map the “branch” concept to JJ bookmarks for logs and status reporting
  (e.g., show active bookmark name where branch name is expected).
- **FR-009**: Provide concise developer help in script usage output documenting JJ
  support and bookmark-based workflow.
- **FR-010**: Ensure idempotency: re-running feature creation with an existing name
  MUST not overwrite; the next number MUST be assigned instead.

### Key Entities *(include if feature involves data)*

- **VCS Context**: Represents the detected SCM and its capabilities
  - Attributes: `type` (`jj` | `git`), `has_git` (bool), `has_jj` (bool)
  - Behavior: branch/bookmark discovery, next-number calculation, create-branch or
    create-bookmark, commit message policy enforcement

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In a JJ-only repo, feature creation completes successfully in under
  3 seconds and produces a bookmark `NNN-short-name` at `@` and a spec directory.
- **SC-002**: In a repo containing both `.jj/` and `.git/`, no Git branch is created
  and JJ is used for all operations (0 Git commands executed).
- **SC-003**: Numbering is correct across 2+ sequential features with the same short
  name; the second feature uses `NNN+1` without overwriting existing artifacts.
- **SC-004**: All automated commits generated by scripts are single-line messages
  with no newline characters.
- **SC-005**: Git-only repos retain existing behavior with no regressions (all
  current workflows pass).
