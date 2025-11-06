#!/usr/bin/env bash
# Common functions and variables for all scripts
#
# VCS Helpers (JJ-first):
# - has_git(): Detects git repo
# - has_jj(): Detects jujutsu repo
# - current_vcs(): Returns 'jj' if .jj/ exists, else 'git'
# - list_feature_markers <short_name>: Lists markers named NNN-<short_name> (bookmarks in JJ, branches in Git)
# - create_feature_marker <full_name>: Creates bookmark (JJ) at @ or branch (Git)

# Get repository root, with fallback for non-git repositories
get_repo_root() {
    if git rev-parse --show-toplevel >/dev/null 2>&1; then
        git rev-parse --show-toplevel
    else
        # Fall back to script location for non-git repos
        local script_dir
        script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        (cd "$script_dir/../../.." && pwd)
    fi
}

# Get current branch, with fallback for non-git repositories
get_current_branch() {
    # First check if SPECIFY_FEATURE environment variable is set
    if [[ -n "${SPECIFY_FEATURE:-}" ]]; then
        echo "$SPECIFY_FEATURE"
        return
    fi

    # Then check git if available
    if git rev-parse --abbrev-ref HEAD >/dev/null 2>&1; then
        git rev-parse --abbrev-ref HEAD
        return
    fi

    # For non-git repos, try to find the latest feature directory
    local repo_root
    repo_root=$(get_repo_root)
    local specs_dir="$repo_root/specs"

    if [[ -d "$specs_dir" ]]; then
        local latest_feature=""
        local highest=0

        for dir in "$specs_dir"/*; do
            if [[ -d "$dir" ]]; then
                local dirname
                dirname=$(basename "$dir")
                if [[ "$dirname" =~ ^([0-9]{3})- ]]; then
                    local number=${BASH_REMATCH[1]}
                    number=$((10#$number))
                    if [[ "$number" -gt "$highest" ]]; then
                        highest=$number
                        latest_feature=$dirname
                    fi
                fi
            fi
        done

        if [[ -n "$latest_feature" ]]; then
            echo "$latest_feature"
            return
        fi
    fi

    echo "main"  # Final fallback
}

# Check if we have git available
has_git() {
    git rev-parse --show-toplevel >/dev/null 2>&1
}

# Check if we have jujutsu available (by repo marker)
has_jj() {
    local repo_root
    repo_root=$(get_repo_root)
    [[ -d "$repo_root/.jj" ]]
}

# Determine preferred VCS (JJ takes precedence if present)
current_vcs() {
    if has_jj; then
        echo jj
    else
        echo git
    fi
}

# List feature markers matching NNN-<short_name>
# Outputs one name per line
list_feature_markers() {
    local short_name="$1"
    local vcs
    vcs=$(current_vcs)
    local repo_root
    repo_root=$(get_repo_root)

    if [[ -z "$short_name" ]]; then
        return 0
    fi

    if [[ "$vcs" == "jj" ]]; then
        # Prefer templated output; fallback to plain list
        if jj bookmark list -T '{name}\n' >/dev/null 2>&1; then
            jj bookmark list -T '{name}\n' 2>/dev/null | grep -E "^[0-9]+-${short_name}$" || true
        else
            jj bookmark list 2>/dev/null | awk '{print $1}' | grep -E "^[0-9]+-${short_name}$" || true
        fi
    else
        # Git: combine remote and local branches
        git ls-remote --heads origin 2>/dev/null | grep -E "refs/heads/[0-9]+-${short_name}$" | sed 's#.*/##' || true
        git branch --list 2>/dev/null | sed 's/^[* ]*//' | grep -E "^[0-9]+-${short_name}$" || true
    fi
}

# Create a feature marker (bookmark in JJ, branch in Git)
create_feature_marker() {
    local name="$1"
    local vcs
    vcs=$(current_vcs)
    if [[ -z "$name" ]]; then
        echo "[specify] ERROR: create_feature_marker requires a name" >&2
        return 1
    fi
    if [[ "$vcs" == "jj" ]]; then
        jj bookmark create "$name" -r @
    else
        git checkout -b "$name"
    fi
}

check_feature_branch() {
    local branch="$1"
    local has_git_repo="$2"
    # If JJ is present, treat bookmark as the branch and skip Git naming enforcement
    if has_jj; then
        echo "[specify] Info: Jujutsu repository detected; skipping Git branch validation" >&2
        return 0
    fi

    # For non-git repos, we can't enforce branch naming but still provide output
    if [[ "$has_git_repo" != "true" ]]; then
        echo "[specify] Warning: Git repository not detected; skipped branch validation" >&2
        return 0
    fi

    if [[ ! "$branch" =~ ^[0-9]{3}- ]]; then
        echo "ERROR: Not on a feature branch. Current branch: $branch" >&2
        echo "Feature branches should be named like: 001-feature-name" >&2
        return 1
    fi

    return 0
}

get_feature_dir() { echo "$1/specs/$2"; }

# Find feature directory by numeric prefix instead of exact branch match
# This allows multiple branches to work on the same spec (e.g., 004-fix-bug, 004-add-feature)
find_feature_dir_by_prefix() {
    local repo_root="$1"
    local branch_name="$2"
    local specs_dir="$repo_root/specs"

    # Extract numeric prefix from branch (e.g., "004" from "004-whatever")
    if [[ ! "$branch_name" =~ ^([0-9]{3})- ]]; then
        # If branch doesn't have numeric prefix, fall back to exact match
        echo "$specs_dir/$branch_name"
        return
    fi

    local prefix="${BASH_REMATCH[1]}"

    # Search for directories in specs/ that start with this prefix
    local matches=()
    if [[ -d "$specs_dir" ]]; then
        for dir in "$specs_dir"/"$prefix"-*; do
            if [[ -d "$dir" ]]; then
                matches+=("$(basename "$dir")")
            fi
        done
    fi

    # Handle results
    if [[ ${#matches[@]} -eq 0 ]]; then
        # No match found - return the branch name path (will fail later with clear error)
        echo "$specs_dir/$branch_name"
    elif [[ ${#matches[@]} -eq 1 ]]; then
        # Exactly one match - perfect!
        echo "$specs_dir/${matches[0]}"
    else
        # Multiple matches - this shouldn't happen with proper naming convention
        echo "ERROR: Multiple spec directories found with prefix '$prefix': ${matches[*]}" >&2
        echo "Please ensure only one spec directory exists per numeric prefix." >&2
        echo "$specs_dir/$branch_name"  # Return something to avoid breaking the script
    fi
}

get_feature_paths() {
    local repo_root
    repo_root=$(get_repo_root)
    local current_branch
    current_branch=$(get_current_branch)
    local has_git_repo="false"

    if has_git; then
        has_git_repo="true"
    fi

    # Use prefix-based lookup to support multiple branches per spec
    local feature_dir
    feature_dir=$(find_feature_dir_by_prefix "$repo_root" "$current_branch")

    cat <<EOF
REPO_ROOT='$repo_root'
CURRENT_BRANCH='$current_branch'
HAS_GIT='$has_git_repo'
FEATURE_DIR='$feature_dir'
FEATURE_SPEC='$feature_dir/spec.md'
IMPL_PLAN='$feature_dir/plan.md'
TASKS='$feature_dir/tasks.md'
RESEARCH='$feature_dir/research.md'
DATA_MODEL='$feature_dir/data-model.md'
QUICKSTART='$feature_dir/quickstart.md'
CONTRACTS_DIR='$feature_dir/contracts'
EOF
}

check_file() { [[ -f "$1" ]] && echo "  ✓ $2" || echo "  ✗ $2"; }
check_dir() { [[ -d "$1" && -n $(ls -A "$1" 2>/dev/null) ]] && echo "  ✓ $2" || echo "  ✗ $2"; }

