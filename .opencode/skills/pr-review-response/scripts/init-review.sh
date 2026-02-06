#!/usr/bin/env bash
# Initialize PR review response session
# Fetches review comments and creates tracking document

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
PR_NUMBER=""
REPO=""
OUTPUT_PATH=""

usage() {
    echo "Usage: $0 <pr-number> [options]"
    echo ""
    echo "Options:"
    echo "  -r, --repo REPO       Override repository (default: current)"
    echo "  -p, --path PATH       Output path for tracking doc (default: .context/pr-review-N.md)"
    echo "  -h, --help            Show this help"
    exit 1
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -r|--repo)
            REPO="$2"
            shift 2
            ;;
        -p|--path)
            OUTPUT_PATH="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            if [[ -z "$PR_NUMBER" ]]; then
                PR_NUMBER="$1"
            else
                echo -e "${RED}Error: Unknown argument: $1${NC}"
                usage
            fi
            shift
            ;;
    esac
done

if [[ -z "$PR_NUMBER" ]]; then
    echo -e "${RED}Error: PR number required${NC}"
    usage
fi

# Get repository from git remote if not specified
if [[ -z "$REPO" ]]; then
    REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "")
    if [[ -z "$REPO" ]]; then
        echo -e "${RED}Error: Could not determine repository. Use --repo option.${NC}"
        exit 1
    fi
fi

# Set default output path
if [[ -z "$OUTPUT_PATH" ]]; then
    OUTPUT_PATH=".context/pr-review-${PR_NUMBER}.md"
fi

echo -e "${BLUE}Initializing PR review response for #${PR_NUMBER}...${NC}"
echo ""

# Fetch review comments
echo -e "${YELLOW}Fetching review comments...${NC}"
COMMENTS=$(gh api "repos/${REPO}/pulls/${PR_NUMBER}/comments" --jq '[.[] | select(.in_reply_to_id == null) | {id: .id, path: .path, line: (.line // .original_line), body: .body}]')

# Count comments
TOTAL=$(echo "$COMMENTS" | jq 'length')

if [[ "$TOTAL" -eq 0 ]]; then
    echo -e "${GREEN}No review comments found on PR #${PR_NUMBER}${NC}"
    exit 0
fi

echo -e "${GREEN}Found ${TOTAL} review comments${NC}"
echo ""

# Categorize comments by priority (simple heuristic based on keywords)
categorize_priority() {
    local body="$1"
    body_lower=$(echo "$body" | tr '[:upper:]' '[:lower:]')
    
    if echo "$body_lower" | grep -qE "(security|vulnerable|exploit|credential|token|secret|injection)"; then
        echo "critical"
    elif echo "$body_lower" | grep -qE "(bug|error|fail|break|incorrect)"; then
        echo "high"
    elif echo "$body_lower" | grep -qE "(refactor|improve|simplify|duplication|deprecat)"; then
        echo "medium"
    else
        echo "low"
    fi
}

# Create output directory
mkdir -p "$(dirname "$OUTPUT_PATH")"

# Generate tracking document
cat > "$OUTPUT_PATH" << EOF
# PR Review Response: PR #${PR_NUMBER}

**Repository:** ${REPO}  
**Generated:** $(date +"%Y-%m-%d %H:%M:%S")  
**Status:** In Progress

## Statistics

EOF

# Count by priority
CRITICAL=$(echo "$COMMENTS" | jq -r '.[] | .body' | while read -r body; do categorize_priority "$body"; done | grep -c "critical" || echo 0)
HIGH=$(echo "$COMMENTS" | jq -r '.[] | .body' | while read -r body; do categorize_priority "$body"; done | grep -c "high" || echo 0)
MEDIUM=$(echo "$COMMENTS" | jq -r '.[] | .body' | while read -r body; do categorize_priority "$body"; done | grep -c "medium" || echo 0)
LOW=$(echo "$COMMENTS" | jq -r '.[] | .body' | while read -r body; do categorize_priority "$body"; done | grep -c "low" || echo 0)

cat >> "$OUTPUT_PATH" << EOF
- **Total comments:** ${TOTAL}
- **Critical:** ${CRITICAL}
- **High:** ${HIGH}
- **Medium:** ${MEDIUM}
- **Low:** ${LOW}
- **Completed:** 0/${TOTAL}

## Progress

- [ ] All critical issues fixed
- [ ] All high priority issues fixed
- [ ] All medium priority issues fixed
- [ ] All low priority issues reviewed
- [ ] All comments replied to
- [ ] Summary comment posted

## Comments

EOF

# Process each priority level
for priority in critical high medium low; do
    priority_upper=$(echo "$priority" | tr '[:lower:]' '[:upper:]')
    cat >> "$OUTPUT_PATH" << EOF

### ${priority_upper} Priority

EOF
    
    # Add comments for this priority
    echo "$COMMENTS" | jq -r '.[] | @json' | while read -r comment; do
        id=$(echo "$comment" | jq -r '.id')
        path=$(echo "$comment" | jq -r '.path')
        line=$(echo "$comment" | jq -r '.line')
        body=$(echo "$comment" | jq -r '.body' | head -c 100)
        
        comment_priority=$(categorize_priority "$(echo "$comment" | jq -r '.body')")
        
        if [[ "$comment_priority" == "$priority" ]]; then
            cat >> "$OUTPUT_PATH" << EOF
- [ ] **${path}:${line}**
  - Comment ID: ${id}
  - Preview: ${body}...
  - Status: pending
  - Commit: _none yet_

EOF
        fi
    done
done

# Add footer
cat >> "$OUTPUT_PATH" << EOF

## Next Steps

1. Review comments by priority (Critical → High → Medium → Low)
2. For each comment:
   - Read and understand the issue
   - Implement fix
   - Test the change
   - Commit with clear message
   - Reply to comment
   - Mark as [x] completed above
3. Push changes incrementally
4. Post summary comment when done

## Resources

- PR: https://github.com/${REPO}/pull/${PR_NUMBER}
- Skill docs: .opencode/skills/pr-review-response/SKILL.md
- Reply script: .opencode/skills/pr-review-response/scripts/reply-comment.sh
- Summary script: .opencode/skills/pr-review-response/scripts/post-summary.sh
EOF

echo -e "${GREEN}✓ Created tracking document: ${OUTPUT_PATH}${NC}"
echo ""
echo -e "${BLUE}Summary:${NC}"
echo -e "  Total: ${TOTAL}"
echo -e "  ${RED}Critical: ${CRITICAL}${NC}"
echo -e "  ${YELLOW}High: ${HIGH}${NC}"
echo -e "  Medium: ${MEDIUM}"
echo -e "  Low: ${LOW}"
echo ""
echo -e "${BLUE}Next: Review ${OUTPUT_PATH} and start addressing comments${NC}"
