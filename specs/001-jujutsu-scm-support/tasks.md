---

description: "Task list for Jujutsu SCM support"
---

# Tasks: Jujutsu SCM Support

**Input**: Design documents from `/specs/001-jujutsu-scm-support/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are OPTIONAL and not requested; we focus on implementation and verification steps.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize shared helpers and developer ergonomics

- [x] T001 Create VCS helper section in .specify/scripts/bash/common.sh (header comments + TODO markers)
- [x] T002 [P] Add helper function stubs in .specify/scripts/bash/common.sh: has_jj, current_vcs, create_feature_marker, list_feature_markers
- [x] T003 Document JJ-first policy in specs/001-jujutsu-scm-support/quickstart.md (link from README.md)
- [x] T004 [P] Add Makefile target `lint-shell` to run shellcheck on .specify/scripts/bash/*.sh (optional local lint; CI integration deferred)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core abstractions that MUST be complete before ANY user story can be implemented

- [x] T005 Implement has_jj() in .specify/scripts/bash/common.sh to detect $REPO_ROOT/.jj
- [x] T006 [P] Implement current_vcs() in .specify/scripts/bash/common.sh to return `jj` if has_jj else `git`
- [x] T007 [P] Implement list_feature_markers() in .specify/scripts/bash/common.sh
      - JJ: parse `jj bookmark list` to collect names matching ^[0-9]{3}-<short-name>$
      - Git: reuse existing branch discovery
- [x] T008 Implement create_feature_marker() in .specify/scripts/bash/common.sh
      - JJ: `jj bookmark create "${name}" -r @`
      - Git: `git checkout -b "${name}"`
- [x] T009 Relax branch validation in .specify/scripts/bash/common.sh:check_feature_branch()
      - If JJ detected (has_jj true), skip Git branch name enforcement even if .git exists

**Checkpoint**: Foundation ready – user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Create Feature in JJ Repo (Priority: P1) 🎯 MVP

**Goal**: Prefer JJ when .jj/ exists, create a bookmark NNN-short-name at @, and scaffold spec dir without invoking Git

**Independent Test**: In a JJ-only repo, run feature creation. Confirm bookmark creation at @ and spec dir created with no Git commands executed.

### Implementation for User Story 1

- [x] T010 [US1] Update .specify/scripts/bash/create-new-feature.sh to use current_vcs() and list_feature_markers()
- [x] T011 [P] [US1] In JJ mode, compute next number from union of JJ bookmarks and specs/ directories (exact short-name match)
- [x] T012 [US1] In JJ mode, create JJ bookmark via create_feature_marker() and skip all Git commands
- [x] T013 [US1] Ensure JSON output remains: BRANCH_NAME, SPEC_FILE, FEATURE_NUM; treat bookmark name as branch name in outputs
- [x] T014 [P] [US1] Update user-facing messages to reference "bookmark" instead of "branch" when in JJ mode

**Checkpoint**: User Story 1 independently delivers JJ-first feature creation

---

## Phase 4: User Story 2 - Numbering and Collision Handling (Priority: P2)

**Goal**: Compute next number correctly using union of JJ bookmarks and specs/ dirs; no overwrites, ensure idempotency

**Independent Test**: Create two features with same short-name; verify second becomes NNN+1 and doesn’t overwrite existing bookmark or dir

### Implementation for User Story 2

- [x] T015 [US2] Extend create-new-feature.sh to guard against overwriting existing bookmark/dir; always select max(N)+1
- [x] T016 [P] [US2] Add strict regex validation for names: ^[0-9]{3}-[a-z0-9]+(-[a-z0-9]+)*$
- [x] T017 [US2] Align spec dir creation to avoid collisions (mkdir -p and fail if path exists); present clear message with next suggestion
- [x] T018 [P] [US2] Add informative logging on numbering sources (bookmarks vs specs) to aid debugging

**Checkpoint**: User Story 2 independently validates numbering across repeated short-names

---

## Phase 5: User Story 3 - Git Fallback Compatibility (Priority: P3)

**Goal**: Preserve Git-only behavior with no regressions

**Independent Test**: In a Git-only repo (no .jj/), ensure the script creates a Git branch and scaffolds spec dir as before

### Implementation for User Story 3

- [x] T019 [US3] Verify Git path in create-new-feature.sh remains unchanged and covered by current_vcs()
- [x] T020 [P] [US3] Ensure setup-plan.sh + common.sh branch checks allow SPECIFY_FEATURE and Git-only workflows
- [x] T021 [US3] Confirm update-agent-context.sh and check-prerequisites.sh resolve feature from SPECIFY_FEATURE or specs dir without JJ requirements

**Checkpoint**: User Story 3 confirms no regressions in Git-only workflows

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T022 [P] Update specs/001-jujutsu-scm-support/quickstart.md with final CLI examples
- [ ] T023 [P] Add troubleshooting notes to README.md linking to quickstart (JJ vs Git, SPECIFY_FEATURE usage)
- [ ] T024 [P] Optional: Add a `bin/lint.sh` wrapper to call `make lint-shell` locally
- [ ] T025 Refactor messages across scripts to ensure consistent terminology (branch vs bookmark)

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1): No dependencies – can start immediately
- Foundational (Phase 2): Depends on Setup completion – BLOCKS all user stories
- User Stories (Phase 3+): All depend on Foundational completion
- Polish (Final Phase): Depends on desired stories being complete

### User Story Dependencies

- User Story 1 (P1): Can start after Foundational – no dependencies on other stories
- User Story 2 (P2): Depends on User Story 1 (extends numbering logic and idempotency)
- User Story 3 (P3): Independent of US2; validates Git fallback compatibility

### Parallel Opportunities

- [P] tasks in Setup and Foundational can run concurrently
- US1 tasks T011 and T014 can run in parallel once T010 starts
- US2 tasks T016 and T018 can run in parallel
- US3 task T020 can run in parallel with T021

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (JJ-first creation)
4. STOP and VALIDATE: Bookmark created at `@`, spec dir exists, no Git commands executed

### Incremental Delivery

1. Add User Story 2 → Validate numbering/idempotency
2. Add User Story 3 → Validate Git fallback
3. Polish & docs
