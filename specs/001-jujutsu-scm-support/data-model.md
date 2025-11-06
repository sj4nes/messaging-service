# Data Model: VCS Context and Feature Markers

Date: 2025-11-05

## Entities

### VCS Context
- Attributes:
  - `type`: `jj` | `git`
  - `has_jj`: boolean (true if `.jj/` exists)
  - `has_git`: boolean (true if `.git/` exists)
- Responsibilities:
  - Determine preferred VCS (`jj` if `.jj/` present)
  - Provide operations for feature marker creation and listing

### Feature Marker
- Definition: The identifier of an active feature work line
- In JJ: a bookmark named `NNN-short-name` at `@`
- In Git: a branch named `NNN-short-name`
- Attributes:
  - `number`: integer (parsed from prefix `NNN`)
  - `short_name`: string (kebab-case)
  - `full_name`: `NNN-short-name`

## Relationships
- VCS Context manages Feature Markers.

## Validation Rules
- `number` is 3 digits, zero-padded; `001`..`999`.
- `short_name` matches `[a-z0-9]+(-[a-z0-9]+)*`.
- `full_name` = `${number}-${short_name}`.
- In JJ, the bookmark must point to `@` at creation time.

## State Transitions
- Create Feature Marker → `new` (created in JJ as bookmark or Git as branch)
- Subsequent features with same short-name → increment `number` (no overwrite)

## Notes
- Remote markers are ignored for numbering to avoid assumptions.
