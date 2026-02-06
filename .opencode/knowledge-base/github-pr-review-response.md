# GitHub PR Review Response Best Practices

**Last Updated:** 2026-02-06  
**Category:** Development Workflow  
**Tags:** github, pr-review, code-review, automation

## Overview

This document captures best practices for systematically addressing GitHub PR review comments using automation and CLI tools.

## Key Principles

### 1. Systematic Approach
- Read ALL comments before starting fixes
- Categorize by priority: Critical → High → Medium → Low
- Create a TODO list to track progress
- Address one issue per commit
- Push incrementally, not all at once

### 2. Clear Communication
- Reply to each review comment with commit reference
- Post a summary comment at PR level (not just inline)
- Use status indicators: ✅ Fixed, ℹ️ Acknowledged, ⚠️ Risk
- Include commit SHA or message in replies
- Explain rationale for "won't fix" decisions

### 3. Commit Strategy
- One logical fix per commit
- Use conventional commit format: `<type>: <description>`
- Types: `security:`, `fix:`, `refactor:`, `chore:`, `docs:`, `test:`
- Reference issue/comment in commit body if needed
- Keep commits focused and reviewable

## GitHub CLI Techniques

### Retrieving Review Comments

```bash
# Get all review comments with details
gh api repos/OWNER/REPO/pulls/PR_NUMBER/comments \
  --jq '.[] | {id: .id, path: .path, line: .line, body: .body[0:200]}'

# Get only unresolved comments
gh api repos/OWNER/REPO/pulls/PR_NUMBER/comments \
  --jq '.[] | select(.in_reply_to_id == null) | {id, path, line, body}'

# Sort by file and line number
gh api repos/OWNER/REPO/pulls/PR_NUMBER/comments \
  --jq '[.[] | select(.in_reply_to_id == null)] | sort_by(.path, .line)'
```

### Replying to Comments

```bash
# Reply to a specific comment
gh api repos/OWNER/REPO/pulls/PR_NUMBER/comments/COMMENT_ID/replies \
  -X POST -f body="✅ Fixed in commit \`<message>\` - <explanation>"

# Post PR-level summary comment (always visible)
gh pr comment PR_NUMBER --body "## Summary of Changes

- ✅ Fixed issue A
- ✅ Fixed issue B
- ℹ️ Acknowledged issue C"
```

### Handling Outdated Comments

When code changes after review:
- Comments on old line numbers become "outdated"
- Replies to outdated comments still work but may be hidden
- **Solution:** Always post a PR-level summary comment
- Include commit references so reviewers can verify fixes

## Security Review Response

### Critical Security Issues
1. **Stop and assess** - Don't rush security fixes
2. **Understand the vulnerability** - Research the attack vector
3. **Fix root cause** - Not just the symptom
4. **Add tests** - Verify the fix prevents the vulnerability
5. **Document rationale** - Explain why the fix works

### Example Security Fixes

```rust
// ❌ Bad: Exposes credentials in error messages
Err(anyhow!("Failed to parse URL: {}", url))

// ✅ Good: Sanitizes credentials before logging
fn sanitize_url(url: &str) -> String {
    url.split('@').last().unwrap_or(url).to_string()
}
Err(anyhow!("Failed to parse URL: {}", sanitize_url(url)))
```

```yaml
# ❌ Bad: Pipes token via stdin (visible in logs if fails)
- run: echo "${{ secrets.TOKEN }}" | gh auth login --with-token

# ✅ Good: Use environment variable (gh CLI auto-detects)
- run: gh pr view 1
  env:
    GH_TOKEN: ${{ secrets.TOKEN }}
```

## Code Quality Review Response

### Deduplication
- Extract common logic into helper functions
- Use single source of truth
- Add tests for the extracted helper

### Simplification
- Make control flow explicit
- Reduce nesting and complexity
- Use early returns
- Add comments for non-obvious logic

### Maintenance
- Update deprecated dependencies
- Remove dead code (with git archaeology to verify)
- Clean up comments and documentation
- Follow language idioms

## Testing Strategy

### Add Tests for Review Comments
- If reviewer suggests adding tests, do it
- Test both happy path and edge cases
- Test error conditions
- Verify fix prevents the original issue

### Test Coverage Additions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_functionality() {
        // Test happy path
        assert_eq!(function("input"), Ok("output"));
        
        // Test edge cases
        assert!(function("").is_err());
        
        // Test error conditions
        assert!(function("invalid").is_err());
    }
}
```

## Documentation Updates

### README Cleanup
- Remove migration notes after completion
- Fix broken badges
- Remove duplicate sections
- Update examples to match current API

### CONTRIBUTING.md Updates
- Document new workflows (e.g., manual release approval)
- Add security review checklists
- Update development setup if changed
- Document CI/CD changes

## Common Pitfalls

### ❌ Don't
- Batch unrelated fixes into one commit
- Reply only to some comments
- Ignore low-priority suggestions without explanation
- Push all fixes at once without incremental testing
- Skip adding tests for fixed bugs

### ✅ Do
- One logical fix per commit
- Reply to ALL comments (even "won't fix")
- Explain rationale for deferring fixes
- Test after each commit
- Push incrementally and verify CI

## Workflow Automation

### Pre-commit Checklist
```bash
# Before committing
cargo test          # Verify tests pass
cargo clippy        # Check for warnings
cargo fmt --check   # Verify formatting
git status          # Review staged changes
```

### Post-push Checklist
```bash
# After pushing
gh pr view PR_NUMBER --web    # Verify changes visible
gh pr checks PR_NUMBER         # Verify CI status
gh pr view PR_NUMBER --json reviews  # Check review status
```

## Reply Templates

### Fixed Issue
```markdown
✅ Fixed in commit `<type>: <message>` - <explanation>

<optional details about the fix>
```

### Acknowledged But Not Fixed
```markdown
ℹ️ Acknowledged - <explanation of why not fixing>

<rationale or context>
```

### Security Issue Fixed
```markdown
🔐 Fixed in commit `security: <message>`

- <what was vulnerable>
- <how it's fixed>  
- <how it's tested>
```

### Testing Added
```markdown
✅ Fixed in commit `test: <message>`

Added tests covering:
- <test case 1>
- <test case 2>
- <test case 3>
```

## Metrics to Track

- Total comments addressed
- Critical/High/Medium/Low breakdown
- Commits created
- Tests added
- Time to resolution
- CI pass rate

## Related Documentation

- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub CLI Manual](https://cli.github.com/manual/)
- [Pull Request Review Guide](https://github.com/features/code-review)
