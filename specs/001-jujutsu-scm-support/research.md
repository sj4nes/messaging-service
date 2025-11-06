# Research: Jujutsu SCM Support

Date: 2025-11-05

## Decisions

1. Prefer JJ when `.jj/` exists (even if `.git/` is present)
   - Rationale: Project policy is JJ-first; avoids Git branch creation in mixed repos.
   - Alternatives: Prompt user; environment flag to force Git. Rejected to keep simple default.

2. Bookmark naming and creation
   - Decision: Use `jj bookmark create NNN-short-name -r @` to bind the bookmark to the current revision.
   - Rationale: Explicitly targets `@`; resilient if default changes.
   - Alternatives: `jj bookmark set` if exists; creating without `-r`; both are less explicit.

3. Bookmark listing for numbering
   - Decision: Use `jj bookmark list` and parse bookmark names matching `^[0-9]{3}-<short-name>$`.
   - Rationale: Purpose-built command; avoids templating complexity.
   - Alternatives: `jj log -T` with `bookmarks` keyword; adds parsing complexity.

4. Numbering sources
   - Decision: Compute next number from union of local JJ bookmarks and `specs/` directories for exact short-name.
   - Rationale: Ensures coherence between VCS marker and docs; avoids remote assumptions.
   - Alternatives: Include remotes; rejected due to variability and no JJ-Git bridge requirement.

5. Commit messages
   - Decision: Scripts must pass single-line messages to `jj commit -m` (no newlines).
   - Rationale: JJ CLI doesn’t support multi-line via `-m` like Git; ensures compatibility.
   - Alternatives: Spawn editor; rejected for automation.

## Implementation Notes

- VCS abstraction to be added to `common.sh`:
  - `has_jj()`: `[ -d "$REPO_ROOT/.jj" ]`
  - `prefer_jj()`: `has_jj && echo true || echo false`
  - `current_vcs()`: outputs `jj` or `git`
  - `create_feature_marker(name)`: if `jj` → `jj bookmark create "$name" -r @`; if `git` → `git checkout -b "$name"`
  - `list_feature_markers(short)`: if `jj` → `jj bookmark list`; if `git` → check branches (existing logic)

- Parsing rules: strict regex `^[0-9]{3}-<short-name>$` for markers and dir names.

## Alternatives Considered

- Dual-write both Git branch and JJ bookmark in mixed repos
  - Rejected: violates JJ-first preference and can cause divergence/confusion.

- Remote bookmark scanning
  - Rejected: out of scope; environments vary.

## Open Questions (resolved by defaults)

- Mixed repo precedence → JJ-first by default
- Error messaging when neither `.jj/` nor `.git/` present → clear guidance to init JJ or Git
