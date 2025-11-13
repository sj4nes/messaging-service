#!/usr/bin/env bash

set -e

JSON_MODE=false
SHORT_NAME=""
BRANCH_NUMBER=""
ARGS=()
i=1
while [ $i -le $# ]; do
    arg="${!i}"
    case "$arg" in
        --json) 
            JSON_MODE=true 
            ;;
        --short-name)
            if [ $((i + 1)) -gt $# ]; then
                echo 'Error: --short-name requires a value' >&2
                exit 1
            fi
            i=$((i + 1))
            next_arg="${!i}"
            # Check if the next argument is another option (starts with --)
            if [[ "$next_arg" == --* ]]; then
                echo 'Error: --short-name requires a value' >&2
                exit 1
            fi
            SHORT_NAME="$next_arg"
            ;;
        --number)
            if [ $((i + 1)) -gt $# ]; then
                echo 'Error: --number requires a value' >&2
                exit 1
            fi
            i=$((i + 1))
            next_arg="${!i}"
            if [[ "$next_arg" == --* ]]; then
                echo 'Error: --number requires a value' >&2
                exit 1
            fi
            BRANCH_NUMBER="$next_arg"
            ;;
        --help|-h) 
            echo "Usage: $0 [--json] [--short-name <name>] [--number N] <feature_description>"
            echo ""
            echo "Options:"
            echo "  --json              Output in JSON format"
            echo "  --short-name <name> Provide a custom short name (2-4 words) for the branch"
            echo "  --number N          Specify branch number manually (overrides auto-detection)"
            echo "  --help, -h          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 'Add user authentication system' --short-name 'user-auth'"
            echo "  $0 'Implement OAuth2 integration for API' --number 5"
            exit 0
            ;;
        *) 
            ARGS+=("$arg") 
            ;;
    esac
    i=$((i + 1))
done

FEATURE_DESCRIPTION="${ARGS[*]}"
if [ -z "$FEATURE_DESCRIPTION" ]; then
    echo "Usage: $0 [--json] [--short-name <name>] [--number N] <feature_description>" >&2
    exit 1
fi

# Function to find the repository root by searching for existing project markers
find_repo_root() {
    local dir="$1"
    while [ "$dir" != "/" ]; do
        if [ -d "$dir/.git" ] || [ -d "$dir/.specify" ] || [ -d "$dir/.jj" ]; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

# Function to check existing branches (local and remote) and return next available number
check_existing_branches() {
    local short_name="$1"
    
    # Fetch all remotes to get latest branch info (suppress errors if no remotes)
    git fetch --all --prune 2>/dev/null || true
    
    # Find all branches matching the pattern using git ls-remote (more reliable)
        local remote_branches
        remote_branches=$(git ls-remote --heads origin 2>/dev/null | grep -E "refs/heads/[0-9]+-${short_name}$" | sed 's#.*/\([0-9]*\)-.*#\1#' | sort -n)
    
    # Also check local branches
        local local_branches
        local_branches=$(git branch 2>/dev/null | grep -E "^[* ]*[0-9]+-${short_name}$" | sed 's/^[* ]*//' | sed 's/-.*//' | sort -n)
    
    # Check specs directory as well
    local spec_dirs=""
    if [ -d "$SPECS_DIR" ]; then
           spec_dirs=$(find "$SPECS_DIR" -maxdepth 1 -type d -name "[0-9]*-${short_name}" -print0 2>/dev/null | xargs -0 -n1 basename 2>/dev/null | sed 's/-.*//' | sort -n)
    fi
    
    # Combine all sources and get the highest number
    local max_num=0
    for num in $remote_branches $local_branches $spec_dirs; do
        if [ "$num" -gt "$max_num" ]; then
            max_num=$num
        fi
    done
    
    # Return next number
    echo $((max_num + 1))
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common helpers
if [ -f "$SCRIPT_DIR/common.sh" ]; then
    # shellcheck source=/dev/null
    . "$SCRIPT_DIR/common.sh"
fi

# Resolve repository root using helper (prefers git if present, else script path)
if command -v git >/dev/null 2>&1 && git rev-parse --show-toplevel >/dev/null 2>&1; then
    REPO_ROOT=$(git rev-parse --show-toplevel)
    HAS_GIT=true
else
    REPO_ROOT="$(find_repo_root "$SCRIPT_DIR")"
    if [ -z "$REPO_ROOT" ]; then
            echo "Error: Could not determine repository root. Please run this script from within the repository." >&2
            exit 1
    fi
    HAS_GIT=false
fi

HAS_JJ=false
if [ -d "$REPO_ROOT/.jj" ]; then HAS_JJ=true; fi

cd "$REPO_ROOT"

SPECS_DIR="$REPO_ROOT/specs"
mkdir -p "$SPECS_DIR"

# Function to generate branch name with stop word filtering and length filtering
generate_branch_name() {
    local description="$1"
    
    # Common stop words to filter out
    local stop_words="^(i|a|an|the|to|for|of|in|on|at|by|with|from|is|are|was|were|be|been|being|have|has|had|do|does|did|will|would|should|could|can|may|might|must|shall|this|that|these|those|my|your|our|their|want|need|add|get|set)$"
    
    # Convert to lowercase and split into words
    local clean_name
    clean_name=$(echo "$description" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/ /g')
    
    # Filter words: remove stop words and words shorter than 3 chars (unless they're uppercase acronyms in original)
    local meaningful_words=()
    for word in $clean_name; do
        # Skip empty words
        [ -z "$word" ] && continue
        
        # Keep words that are NOT stop words AND (length >= 3 OR are potential acronyms)
        if ! echo "$word" | grep -qiE "$stop_words"; then
            if [ ${#word} -ge 3 ]; then
                meaningful_words+=("$word")
            elif echo "$description" | grep -q "\b${word^^}\b"; then
                # Keep short words if they appear as uppercase in original (likely acronyms)
                meaningful_words+=("$word")
            fi
        fi
    done
    
    # If we have meaningful words, use first 3-4 of them
    if [ ${#meaningful_words[@]} -gt 0 ]; then
        local max_words=3
        if [ ${#meaningful_words[@]} -eq 4 ]; then max_words=4; fi
        
        local result=""
        local count=0
        for word in "${meaningful_words[@]}"; do
            if [ $count -ge $max_words ]; then break; fi
            if [ -n "$result" ]; then result="$result-"; fi
            result="$result$word"
            count=$((count + 1))
        done
        echo "$result"
    else
        # Fallback to original logic if no meaningful words found
        echo "$description" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/-\+/-/g' | sed 's/^-//' | sed 's/-$//' | tr '-' '\n' | grep -v '^$' | head -3 | tr '\n' '-' | sed 's/-$//'
    fi
}

# Generate branch name
if [ -n "$SHORT_NAME" ]; then
    # Use provided short name, just clean it up
    BRANCH_SUFFIX=$(echo "$SHORT_NAME" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/-\+/-/g' | sed 's/^-//' | sed 's/-$//')
else
    # Generate from description with smart filtering
    BRANCH_SUFFIX=$(generate_branch_name "$FEATURE_DESCRIPTION")
fi

get_next_number_for_short_name() {
    local short_name="$1"
    local max_num=0

        # JJ bookmarks
        if [ "$HAS_JJ" = true ]; then
            local n
            if jj bookmark list -T '{name}\n' >/dev/null 2>&1; then
                n=$(jj bookmark list -T '{name}\n' 2>/dev/null | grep -E "^[0-9]+-${short_name}$" | sed 's/-.*//' | sort -n | tail -1)
            else
                n=$(jj bookmark list 2>/dev/null | awk '{print $1}' | grep -E "^[0-9]+-${short_name}$" | sed 's/-.*//' | sort -n | tail -1)
            fi
            if [ -n "$n" ] && [ "$n" -gt "$max_num" ]; then max_num=$n; fi
        fi

        # Git branches (remote + local)
        if [ "$HAS_GIT" = true ]; then
            local n
            n=$(git ls-remote --heads origin 2>/dev/null | grep -E "refs/heads/[0-9]+-${short_name}$" | sed 's#.*/##' | sed 's/-.*//' | sort -n | tail -1)
            if [ -n "$n" ] && [ "$n" -gt "$max_num" ]; then max_num=$n; fi
            n=$(git branch 2>/dev/null | sed 's/^[* ]*//' | grep -E "^[0-9]+-${short_name}$" | sed 's/-.*//' | sort -n | tail -1)
            if [ -n "$n" ] && [ "$n" -gt "$max_num" ]; then max_num=$n; fi
        fi

        # specs directories
        if [ -d "$SPECS_DIR" ]; then
            local n
            n=$(find "$SPECS_DIR" -maxdepth 1 -type d -name "[0-9][0-9][0-9]-${short_name}" -exec basename {} \; | sed 's/-.*//' | sort -n | tail -1)
            if [ -n "$n" ] && [ "$n" -gt "$max_num" ]; then max_num=$n; fi
        fi

    echo $((max_num + 1))
}

get_next_global_number() {
    # Returns next global feature number ignoring short name.
    local max_num=0

    # JJ bookmarks
    if [ "$HAS_JJ" = true ]; then
        local jj_list
        if jj bookmark list -T '{name}\n' >/dev/null 2>&1; then
            jj_list=$(jj bookmark list -T '{name}\n' 2>/dev/null)
        else
            jj_list=$(jj bookmark list 2>/dev/null | awk '{print $1}')
        fi
        while IFS= read -r line; do
            local num
            num=$(echo "$line" | grep -E '^[0-9]{3}-' | sed 's/-.*//') || true
            if [ -n "$num" ] && [ "$num" -gt "$max_num" ]; then max_num=$num; fi
        done <<<"$jj_list"
    fi

    # Git remote branches
    if [ "$HAS_GIT" = true ]; then
        local remote
        remote=$(git ls-remote --heads origin 2>/dev/null | grep -E 'refs/heads/[0-9]{3}-' | sed 's#.*/##') || true
        while IFS= read -r line; do
            local num
            num=$(echo "$line" | sed 's/-.*//' )
            if [ -n "$num" ] && [ "$num" -gt "$max_num" ]; then max_num=$num; fi
        done <<<"$remote"

        # Git local branches
        local localb
        localb=$(git branch 2>/dev/null | sed 's/^[* ]*//' | grep -E '^[0-9]{3}-') || true
        while IFS= read -r line; do
            local num
            num=$(echo "$line" | sed 's/-.*//')
            if [ -n "$num" ] && [ "$num" -gt "$max_num" ]; then max_num=$num; fi
        done <<<"$localb"
    fi

    # Specs directories
    if [ -d "$SPECS_DIR" ]; then
        local specs
        specs=$(find "$SPECS_DIR" -maxdepth 1 -type d -name '[0-9][0-9][0-9]-*' -exec basename {} \; 2>/dev/null) || true
        while IFS= read -r line; do
            local num
            num=$(echo "$line" | sed 's/-.*//')
            if [ -n "$num" ] && [ "$num" -gt "$max_num" ]; then max_num=$num; fi
        done <<<"$specs"
    fi

    echo $((max_num + 1))
}

# Determine branch number (global, ignores short name)
if [ -z "$BRANCH_NUMBER" ]; then
    BRANCH_NUMBER=$(get_next_global_number)
fi

FEATURE_NUM=$(printf "%03d" "$BRANCH_NUMBER")
BRANCH_NAME="${FEATURE_NUM}-${BRANCH_SUFFIX}"

# Safety: auto-increment if branch/bookmark/spec directory already exists.
collision_exists() {
    local name="$1"
    # JJ bookmark
    if [ "$HAS_JJ" = true ] && jj bookmark list 2>/dev/null | awk '{print $1}' | grep -q "^${name}$"; then return 0; fi
    # Git local branch
    if [ "$HAS_GIT" = true ] && git branch 2>/dev/null | sed 's/^[* ]*//' | grep -q "^${name}$"; then return 0; fi
    # Git remote branch
    if [ "$HAS_GIT" = true ] && git ls-remote --heads origin 2>/dev/null | awk '{print $2}' | sed 's#refs/heads/##' | grep -q "^${name}$"; then return 0; fi
    # Spec directory
    if [ -d "$SPECS_DIR/${name}" ]; then return 0; fi
    return 1
}

loop_guard=0
while ! collision_exists "$BRANCH_NAME"; do
    break # no collision, exit loop
done
while collision_exists "$BRANCH_NAME"; do
    loop_guard=$((loop_guard+1))
    if [ $loop_guard -gt 50 ]; then
        >&2 echo "[specify] ERROR: collision avoidance exceeded 50 iterations; aborting."; exit 1
    fi
    BRANCH_NUMBER=$((BRANCH_NUMBER+1))
    FEATURE_NUM=$(printf "%03d" "$BRANCH_NUMBER")
    BRANCH_NAME="${FEATURE_NUM}-${BRANCH_SUFFIX}"
done

# GitHub enforces a 244-byte limit on branch names
# Validate and truncate if necessary
MAX_BRANCH_LENGTH=244
if [ ${#BRANCH_NAME} -gt $MAX_BRANCH_LENGTH ]; then
    # Calculate how much we need to trim from suffix
    # Account for: feature number (3) + hyphen (1) = 4 chars
    MAX_SUFFIX_LENGTH=$((MAX_BRANCH_LENGTH - 4))
    
    # Truncate suffix at word boundary if possible
    TRUNCATED_SUFFIX=$(echo "$BRANCH_SUFFIX" | cut -c1-$MAX_SUFFIX_LENGTH)
    # Remove trailing hyphen if truncation created one
    TRUNCATED_SUFFIX=${TRUNCATED_SUFFIX%-}
    
    ORIGINAL_BRANCH_NAME="$BRANCH_NAME"
    BRANCH_NAME="${FEATURE_NUM}-${TRUNCATED_SUFFIX}"
    
    >&2 echo "[specify] Warning: Branch name exceeded GitHub's 244-byte limit"
    >&2 echo "[specify] Original: $ORIGINAL_BRANCH_NAME (${#ORIGINAL_BRANCH_NAME} bytes)"
    >&2 echo "[specify] Truncated to: $BRANCH_NAME (${#BRANCH_NAME} bytes)"
fi

# Validate final name format strictly (NNN-short-name)
if ! echo "$BRANCH_NAME" | grep -Eq '^[0-9]{3}-[a-z0-9]+(-[a-z0-9]+)*$'; then
    >&2 echo "[specify] ERROR: Generated feature name '$BRANCH_NAME' is invalid. Expected 'NNN-short-name' with lowercase alphanumerics and hyphens."
    exit 1
fi

# Debug/log numbering sources (stderr) when not JSON
if ! $JSON_MODE; then
    >&2 echo "[specify] Using VCS: $([ "$HAS_JJ" = true ] && echo JJ || echo ${HAS_GIT:+Git})"
    >&2 echo "[specify] Selected feature number: $FEATURE_NUM for short-name: $BRANCH_SUFFIX"
fi

# Create feature marker (JJ bookmark or Git branch)
if [ "$HAS_JJ" = true ]; then
    if ! jj bookmark list -T '{name}\n' 2>/dev/null | grep -q "^${BRANCH_NAME}$"; then
        jj bookmark create "$BRANCH_NAME" -r @
        echo "[specify] Created JJ bookmark: $BRANCH_NAME at @" >&2
    else
        echo "[specify] Info: JJ bookmark already exists: $BRANCH_NAME" >&2
    fi
elif [ "$HAS_GIT" = true ]; then
    git checkout -b "$BRANCH_NAME"
else
    >&2 echo "[specify] Warning: No VCS detected; skipped marker creation for $BRANCH_NAME"
fi

FEATURE_DIR="$SPECS_DIR/$BRANCH_NAME"
mkdir -p "$FEATURE_DIR"

TEMPLATE="$REPO_ROOT/.specify/templates/spec-template.md"
SPEC_FILE="$FEATURE_DIR/spec.md"
if [ -f "$TEMPLATE" ]; then cp "$TEMPLATE" "$SPEC_FILE"; else touch "$SPEC_FILE"; fi

# Set the SPECIFY_FEATURE environment variable for the current session
export SPECIFY_FEATURE="$BRANCH_NAME"

if $JSON_MODE; then
    printf '{"BRANCH_NAME":"%s","SPEC_FILE":"%s","FEATURE_NUM":"%s"}\n' "$BRANCH_NAME" "$SPEC_FILE" "$FEATURE_NUM"
else
    echo "BRANCH_NAME: $BRANCH_NAME"
    echo "SPEC_FILE: $SPEC_FILE"
    echo "FEATURE_NUM: $FEATURE_NUM"
    echo "SPECIFY_FEATURE environment variable set to: $BRANCH_NAME"
fi
