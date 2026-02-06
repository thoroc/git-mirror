# PR Review Response Skill

**Version:** 1.0.0  
**Category:** Development Workflow  
**Tags:** github, pr-review, automation, workflow

## Overview

Systematic approach to addressing GitHub PR review comments with automation, tracking, and best practices.

## When to Use This Skill

Use this skill when:
- You receive review comments on a GitHub PR
- You need to systematically address multiple feedback items
- You want to ensure all comments are replied to
- You need to track progress on fixes

## Prerequisites

- GitHub CLI (`gh`) installed and authenticated
- Repository with an open pull request
- Review comments on the PR
- Git working directory clean

## Workflow

### 1. Initialize Review Response Session

```bash
# Run the initialization script
.opencode/skills/pr-review-response/scripts/init-review.sh PR_NUMBER

# This will:
# - Fetch all review comments
# - Create a TODO list in .context/pr-review-PR_NUMBER.md
# - Categorize comments by priority
# - Show summary statistics
```

### 2. Address Comments Systematically

For each comment:

1. **Read the comment** - Understand the issue
2. **Assess priority** - Critical > High > Medium > Low
3. **Implement fix** - Make the code change
4. **Test the fix** - Run tests, verify behavior
5. **Commit with clear message** - Use conventional commit format
6. **Mark TODO complete** - Update tracking document

### 3. Reply to Comments

```bash
# After fixing, reply to the comment
.opencode/skills/pr-review-response/scripts/reply-comment.sh \
  PR_NUMBER \
  COMMENT_ID \
  "✅ Fixed in commit \`<message>\` - <explanation>"
```

### 4. Post Summary

```bash
# When all fixes are done
.opencode/skills/pr-review-response/scripts/post-summary.sh PR_NUMBER

# This will:
# - Generate summary of all fixes
# - Post comprehensive comment to PR
# - Include commit references
# - Show test results
```

## Commit Message Format

Use conventional commits:

```
<type>: <description>

[optional body]
[optional footer]
```

### Types
- `security:` - Security vulnerability fixes
- `fix:` - Bug fixes
- `refactor:` - Code improvements without behavior change
- `chore:` - Maintenance tasks (deps, CI, etc.)
- `docs:` - Documentation updates
- `test:` - Test additions or fixes
- `perf:` - Performance improvements

### Examples

```bash
# Security fix
git commit -m "security: sanitize URLs with credentials in error messages

- Add sanitize_url() function to strip credentials
- Apply to all error logging
- Add comprehensive tests"

# Code quality fix
git commit -m "refactor: extract common helper to remove duplication

- Create extract_full_host() helper
- Use in get_host_from_repo() and get_host_from_repo_full()
- Reduces duplication by 30 lines"

# Documentation fix
git commit -m "docs: remove migration meta-commentary from README

- Remove unprofessional migration notes
- Clean up for production readiness"
```

## Reply Templates

### Fixed Issue
```markdown
✅ Fixed in commit `<type>: <message>` - <explanation>

<optional: details about the fix>
<optional: link to commit if helpful>
```

### Security Issue Fixed
```markdown
🔐 Fixed in commit `security: <message>`

**What was vulnerable:** <description>
**How it's fixed:** <explanation>
**How it's tested:** <test description>
```

### Won't Fix (with Rationale)
```markdown
ℹ️ Acknowledged - <explanation of why not fixing>

<rationale or context>
<alternative approach if applicable>
```

### Test Coverage Added
```markdown
✅ Fixed in commit `test: <message>`

Added tests covering:
- <test case 1>
- <test case 2>
- <test case 3>

All tests passing: X/X
```

## Best Practices

### Do's ✅
- Address comments in priority order (Critical → Low)
- One logical fix per commit
- Test after every change
- Reply to ALL comments (even "won't fix")
- Push incrementally, verify CI passes
- Post PR-level summary comment

### Don'ts ❌
- Batch unrelated fixes into one commit
- Skip adding tests for fixed bugs
- Ignore low-priority suggestions without explanation
- Push all fixes at once without testing
- Reply only to some comments

## Progress Tracking

The skill maintains a tracking document at `.context/pr-review-PR_NUMBER.md`:

```markdown
# PR Review Response: PR #123

## Statistics
- Total comments: 21
- Critical: 3
- High: 5
- Medium: 8
- Low: 5
- Completed: 0/21

## Comments

### Critical Priority

- [ ] **security: URL credential exposure** (util.rs:51)
  - Comment ID: 2760500873
  - Status: pending
  - Commit: _none yet_

### High Priority
...
```

Update this document as you progress through fixes.

## Scripts Reference

### init-review.sh
Fetches review comments and creates tracking document.

**Usage:**
```bash
.opencode/skills/pr-review-response/scripts/init-review.sh <pr-number>
```

**Options:**
- `-r, --repo` - Override repository (default: current)
- `-p, --path` - Output path for tracking doc

### reply-comment.sh
Posts a reply to a specific review comment.

**Usage:**
```bash
.opencode/skills/pr-review-response/scripts/reply-comment.sh <pr-number> <comment-id> <message>
```

**Options:**
- `-r, --repo` - Override repository
- `-f, --file` - Read message from file

### post-summary.sh
Posts comprehensive summary comment to PR.

**Usage:**
```bash
.opencode/skills/pr-review-response/scripts/post-summary.sh <pr-number>
```

**Options:**
- `-r, --repo` - Override repository
- `-i, --input` - Path to tracking document

## Examples

### Complete Workflow Example

```bash
# 1. Initialize
cd /path/to/repo
.opencode/skills/pr-review-response/scripts/init-review.sh 123

# Review shows 21 comments
# Edit .context/pr-review-123.md to plan approach

# 2. Fix first critical issue
# ... make code changes ...
cargo test
git add src/util.rs
git commit -m "security: sanitize URLs with credentials in error messages"
git push

# 3. Reply to comment
.opencode/skills/pr-review-response/scripts/reply-comment.sh 123 2760500873 \
  "✅ Fixed in commit \`security: sanitize URLs...\` - Added sanitize_url() function"

# 4. Mark complete in tracking doc
# Edit .context/pr-review-123.md and mark as [x]

# 5. Repeat for all comments

# 6. Post final summary
.opencode/skills/pr-review-response/scripts/post-summary.sh 123
```

## Troubleshooting

### Comment replies not visible
**Issue:** Replies appear under "outdated" sections  
**Cause:** Code changes moved the line numbers  
**Solution:** Always post PR-level summary comment (not just inline replies)

### CI failures after push
**Issue:** Tests fail in CI but pass locally  
**Cause:** Environment differences or race conditions  
**Solution:** Run full test suite locally, check CI logs for specifics

### Can't reply to comment
**Issue:** GitHub API returns 404 or 422  
**Cause:** Comment may be outdated or on deleted code  
**Solution:** Skip the inline reply, mention it in PR-level summary instead

## Related Skills

- `conventional-commits` - Commit message formatting
- `git-workflow` - Git best practices
- `ci-debugging` - Troubleshooting CI failures

## Related Documentation

- `.opencode/knowledge-base/github-pr-review-response.md` - Best practices guide
- `.opencode/knowledge-base/rust-security-review.md` - Security review checklist
