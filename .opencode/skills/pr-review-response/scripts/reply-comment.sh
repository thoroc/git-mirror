#!/usr/bin/env bash
# Reply to a specific PR review comment

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
PR_NUMBER=""
COMMENT_ID=""
MESSAGE=""
REPO=""
MESSAGE_FILE=""

usage() {
    echo "Usage: $0 <pr-number> <comment-id> <message> [options]"
    echo ""
    echo "Options:"
    echo "  -r, --repo REPO       Override repository (default: current)"
    echo "  -f, --file FILE       Read message from file"
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
        -f|--file)
            MESSAGE_FILE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            if [[ -z "$PR_NUMBER" ]]; then
                PR_NUMBER="$1"
            elif [[ -z "$COMMENT_ID" ]]; then
                COMMENT_ID="$1"
            elif [[ -z "$MESSAGE" ]]; then
                MESSAGE="$1"
            else
                echo -e "${RED}Error: Unknown argument: $1${NC}"
                usage
            fi
            shift
            ;;
    esac
done

# Validation
if [[ -z "$PR_NUMBER" ]] || [[ -z "$COMMENT_ID" ]]; then
    echo -e "${RED}Error: PR number and comment ID required${NC}"
    usage
fi

# Read message from file if specified
if [[ -n "$MESSAGE_FILE" ]]; then
    if [[ ! -f "$MESSAGE_FILE" ]]; then
        echo -e "${RED}Error: Message file not found: ${MESSAGE_FILE}${NC}"
        exit 1
    fi
    MESSAGE=$(cat "$MESSAGE_FILE")
elif [[ -z "$MESSAGE" ]]; then
    echo -e "${RED}Error: Message required (or use --file)${NC}"
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

echo -e "${BLUE}Replying to comment ${COMMENT_ID} on PR #${PR_NUMBER}...${NC}"

# Post reply
if gh api "repos/${REPO}/pulls/${PR_NUMBER}/comments/${COMMENT_ID}/replies" \
    -X POST \
    -f body="$MESSAGE" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Reply posted successfully${NC}"
    echo ""
    echo -e "${BLUE}View at: https://github.com/${REPO}/pull/${PR_NUMBER}#discussion_r${COMMENT_ID}${NC}"
else
    echo -e "${RED}✗ Failed to post reply${NC}"
    echo -e "${RED}This may happen if the comment is on outdated code.${NC}"
    echo -e "${RED}Consider adding to PR-level summary comment instead.${NC}"
    exit 1
fi
