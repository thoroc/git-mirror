# GitHub PR Review Response Best Practices

**Last Updated:** 2026-02-06  
**Category:** Development Workflow  
**Tags:** github, pr-review, code-review, automation, language-agnostic

## Overview

This document captures best practices for systematically addressing GitHub PR review comments using automation and CLI tools. These practices apply to projects in any programming language.

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

### Common Security Patterns

#### Credential Exposure

```javascript
// ❌ Bad: Exposes credentials in logs
console.error(`Failed to connect: ${connectionString}`);

// ✅ Good: Sanitizes before logging
function sanitizeUrl(url) {
  return url.split('@').pop();
}
console.error(`Failed to connect: ${sanitizeUrl(connectionString)}`);
```

#### Token Handling in CI

```yaml
# ❌ Bad: Token in command line (visible in process list)
- run: curl -H "Authorization: token ${{ secrets.TOKEN }}" ...

# ✅ Good: Token in environment variable
- run: curl -H "Authorization: token $TOKEN" ...
  env:
    TOKEN: ${{ secrets.TOKEN }}
```

#### Path Traversal

```python
# ❌ Bad: User input directly in path
file_path = os.path.join(base_dir, user_input)

# ✅ Good: Validate and sanitize input
import os.path
safe_name = os.path.basename(user_input)  # Remove directory parts
file_path = os.path.join(base_dir, safe_name)
```

#### Command Injection

```python
# ❌ Bad: Shell=True with user input
subprocess.run(f"git clone {repo_url}", shell=True)

# ✅ Good: Pass as array, no shell
subprocess.run(["git", "clone", repo_url], shell=False)
```

## Code Quality Review Response

### Deduplication
- Extract common logic into helper functions
- Use single source of truth
- Add tests for the extracted helper
- Document the shared functionality

### Simplification
- Make control flow explicit
- Reduce nesting and complexity
- Use early returns
- Add comments for non-obvious logic
- Follow language-specific idioms

### Maintenance
- Update deprecated dependencies
- Remove dead code (with git archaeology to verify)
- Clean up comments and documentation
- Modernize patterns to current best practices

## Testing Strategy

### Add Tests for Review Comments
- If reviewer suggests adding tests, do it
- Test both happy path and edge cases
- Test error conditions
- Verify fix prevents the original issue

### Language-Specific Examples

**JavaScript/TypeScript:**
```javascript
describe('function name', () => {
  it('should handle valid input', () => {
    expect(function('input')).toBe('output');
  });
  
  it('should handle edge cases', () => {
    expect(function('')).toThrow();
  });
});
```

**Python:**
```python
def test_function_valid_input():
    assert function('input') == 'output'

def test_function_edge_case():
    with pytest.raises(ValueError):
        function('')
```

**Java:**
```java
@Test
public void testFunctionValidInput() {
    assertEquals("output", function("input"));
}

@Test(expected = IllegalArgumentException.class)
public void testFunctionEdgeCase() {
    function("");
}
```

## Documentation Updates

### README Cleanup
- Remove migration notes after completion
- Fix broken badges
- Remove duplicate sections
- Update examples to match current API
- Check all links are valid

### CONTRIBUTING.md Updates
- Document new workflows (e.g., manual release approval)
- Add security review checklists
- Update development setup if changed
- Document CI/CD changes
- Update branch strategy if modified

## Common Pitfalls

### ❌ Don't
- Batch unrelated fixes into one commit
- Reply only to some comments
- Ignore low-priority suggestions without explanation
- Push all fixes at once without incremental testing
- Skip adding tests for fixed bugs
- Use language-specific jargon in commit messages

### ✅ Do
- One logical fix per commit
- Reply to ALL comments (even "won't fix")
- Explain rationale for deferring fixes
- Test after each commit
- Push incrementally and verify CI
- Use clear, descriptive commit messages

## Workflow Automation

### Pre-commit Checklist

**Language-agnostic:**
```bash
git status          # Review staged changes
git diff --cached   # Review actual changes
```

**Language-specific examples:**
```bash
# JavaScript/TypeScript
npm test
npm run lint
npm run format:check

# Python
pytest
pylint src/
black --check .

# Rust
cargo test
cargo clippy
cargo fmt --check

# Go
go test ./...
go vet ./...
gofmt -l .

# Java
mvn test
mvn checkstyle:check
```

### Post-push Checklist
```bash
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

- **What was vulnerable:** <description>
- **How it's fixed:** <explanation>  
- **How it's tested:** <test description>
```

### Testing Added
```markdown
✅ Fixed in commit `test: <message>`

Added tests covering:
- <test case 1>
- <test case 2>
- <test case 3>

All tests passing: X/X
```

### Refactoring/Code Quality
```markdown
✅ Fixed in commit `refactor: <message>`

**Changes made:**
- <change 1>
- <change 2>

**Benefits:**
- <benefit 1>
- <benefit 2>

All tests still passing: X/X
```

## Metrics to Track

- Total comments addressed
- Critical/High/Medium/Low breakdown
- Commits created
- Tests added
- Lines of code changed
- Time to resolution
- CI pass rate

## Related Documentation

- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub CLI Manual](https://cli.github.com/manual/)
- [Pull Request Review Guide](https://github.com/features/code-review)
- Language-specific security guides (OWASP, etc.)
