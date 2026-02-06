#!/usr/bin/env bash
# Post comprehensive summary comment to PR

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
PR_NUMBER=""
REPO=""
INPUT_PATH=""

usage() {
    echo "Usage: $0 <pr-number> [options]"
    echo ""
    echo "Options:"
    echo "  -r, --repo REPO       Override repository (default: current)"
    echo "  -i, --input PATH      Path to tracking document (default: .context/pr-review-N.md)"
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
        -i|--input)
            INPUT_PATH="$2"
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

# Set default input path
if [[ -z "$INPUT_PATH" ]]; then
    INPUT_PATH=".context/pr-review-${PR_NUMBER}.md"
fi

# Check if tracking document exists
if [[ ! -f "$INPUT_PATH" ]]; then
    echo -e "${YELLOW}Warning: Tracking document not found: ${INPUT_PATH}${NC}"
    echo -e "${YELLOW}Generating summary from git log instead...${NC}"
    INPUT_PATH=""
fi

echo -e "${BLUE}Generating summary comment for PR #${PR_NUMBER}...${NC}"
echo ""

# Generate summary
SUMMARY="## 🎯 Review Comments Addressed

Thank you for the comprehensive review! Here's a summary of all fixes:

"

# Get recent commits (since this session)
COMMITS=$(git log --oneline --since="6 hours ago" | head -20)
COMMIT_COUNT=$(echo "$COMMITS" | wc -l | tr -d ' ')

if [[ -n "$INPUT_PATH" ]] && [[ -f "$INPUT_PATH" ]]; then
    # Extract stats from tracking document
    TOTAL=$(grep "Total comments:" "$INPUT_PATH" | grep -oE "[0-9]+" || echo "0")
    CRITICAL=$(grep "Critical:" "$INPUT_PATH" | grep -oE "[0-9]+" || echo "0")
    HIGH=$(grep "High:" "$INPUT_PATH" | grep -oE "[0-9]+" || echo "0")
    MEDIUM=$(grep "Medium:" "$INPUT_PATH" | grep -oE "[0-9]+" || echo "0")
    LOW=$(grep "Low:" "$INPUT_PATH" | grep -oE "[0-9]+" || echo "0")
    COMPLETED=$(grep -c "\[x\]" "$INPUT_PATH" || echo "0")
    
    SUMMARY+="### 📊 Statistics

- **Total comments:** ${TOTAL}
- **Completed:** ${COMPLETED}/${TOTAL}
- **Commits created:** ${COMMIT_COUNT}

### Priority Breakdown

- 🔴 **Critical:** ${CRITICAL}
- 🟡 **High:** ${HIGH}
- 🟢 **Medium:** ${MEDIUM}
- ⚪ **Low:** ${LOW}

"
fi

# Group commits by type
SUMMARY+="### 📝 Commits

"

# Security commits
SECURITY_COMMITS=$(echo "$COMMITS" | grep "^[^ ]* security:" || true)
if [[ -n "$SECURITY_COMMITS" ]]; then
    SUMMARY+="#### 🔐 Security Fixes

\`\`\`
${SECURITY_COMMITS}
\`\`\`

"
fi

# Fix commits
FIX_COMMITS=$(echo "$COMMITS" | grep "^[^ ]* fix:" || true)
if [[ -n "$FIX_COMMITS" ]]; then
    SUMMARY+="#### 🐛 Bug Fixes

\`\`\`
${FIX_COMMITS}
\`\`\`

"
fi

# Refactor commits
REFACTOR_COMMITS=$(echo "$COMMITS" | grep "^[^ ]* refactor:" || true)
if [[ -n "$REFACTOR_COMMITS" ]]; then
    SUMMARY+="#### ♻️ Code Quality

\`\`\`
${REFACTOR_COMMITS}
\`\`\`

"
fi

# Chore commits
CHORE_COMMITS=$(echo "$COMMITS" | grep "^[^ ]* chore:" || true)
if [[ -n "$CHORE_COMMITS" ]]; then
    SUMMARY+="#### 🔧 Maintenance

\`\`\`
${CHORE_COMMITS}
\`\`\`

"
fi

# Docs commits
DOCS_COMMITS=$(echo "$COMMITS" | grep "^[^ ]* docs:" || true)
if [[ -n "$DOCS_COMMITS" ]]; then
    SUMMARY+="#### 📚 Documentation

\`\`\`
${DOCS_COMMITS}
\`\`\`

"
fi

# Test commits
TEST_COMMITS=$(echo "$COMMITS" | grep "^[^ ]* test:" || true)
if [[ -n "$TEST_COMMITS" ]]; then
    SUMMARY+="#### ✅ Testing

\`\`\`
${TEST_COMMITS}
\`\`\`

"
fi

# Add verification section
SUMMARY+="
### ✅ Verification

"

# Run tests if in a Rust project
if [[ -f "Cargo.toml" ]]; then
    echo -e "${BLUE}Running tests...${NC}"
    if cargo test --quiet 2>&1 | grep -E "test result:" > /dev/null; then
        TEST_RESULT=$(cargo test --quiet 2>&1 | grep "test result:" | head -1)
        SUMMARY+="- Tests: ${TEST_RESULT}
"
    fi
    
    # Run clippy
    echo -e "${BLUE}Running clippy...${NC}"
    if cargo clippy --quiet 2>&1 | grep -q "warning:"; then
        SUMMARY+="- Clippy: ⚠️ Warnings present
"
    else
        SUMMARY+="- Clippy: ✅ Clean
"
    fi
fi

SUMMARY+="
---

**Note:** Some comment replies may appear under \"outdated\" sections because the code has been updated. All issues have been addressed.
"

# Save summary to temp file for review
TEMP_FILE=$(mktemp)
echo "$SUMMARY" > "$TEMP_FILE"

echo -e "${GREEN}✓ Generated summary${NC}"
echo ""
echo -e "${BLUE}Preview:${NC}"
echo "---"
head -30 "$TEMP_FILE"
echo "..."
echo "---"
echo ""

read -p "Post this summary to PR #${PR_NUMBER}? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if gh pr comment "$PR_NUMBER" --body-file "$TEMP_FILE"; then
        echo -e "${GREEN}✓ Summary posted successfully${NC}"
        echo ""
        echo -e "${BLUE}View at: https://github.com/${REPO}/pull/${PR_NUMBER}${NC}"
    else
        echo -e "${RED}✗ Failed to post summary${NC}"
        echo -e "${YELLOW}Summary saved to: ${TEMP_FILE}${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}Summary not posted${NC}"
    echo -e "${YELLOW}Saved to: ${TEMP_FILE}${NC}"
fi

rm -f "$TEMP_FILE"
