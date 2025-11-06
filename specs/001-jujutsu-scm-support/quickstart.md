# Quickstart: Jujutsu SCM Support

This feature makes spec-kit scripts work seamlessly in Jujutsu (JJ) repositories.

## Prerequisites
- Jujutsu (`jj`) installed
- Repository has a `.jj/` directory (JJ-initialized)

## Create a new feature (JJ-first)

- Run your spec-kit command to create a feature (e.g., `/speckit.specify`).
- The scripts will:
  - Prefer JJ when `.jj/` exists
  - Create a bookmark at `@` named `NNN-short-name`
  - Scaffold `specs/NNN-short-name/` with `spec.md`

## Numbering
- The next feature number is computed from the union of:
  - JJ bookmarks named `NNN-short-name`
  - Directories under `specs/` named `NNN-short-name`
- Remotes are not scanned.

## Commit messages
- Automated commits use single-line messages with `jj commit -m`.
- If you need multi-line messages, open an editor (manual flow).

## Mixed repos (JJ + Git)
- JJ is preferred by default; only a JJ bookmark is created.
- Git-only repositories continue with the existing branch workflow.

## Troubleshooting
- "Not on a feature branch" during planning: Set `SPECIFY_FEATURE=NNN-short-name` to help scripts locate the feature directory when Git is in detached HEAD.
- No `.jj/` and no `.git/`: initialize repository with `jj init` or `git init`.
