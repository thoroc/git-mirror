# Rust Security Review Checklist

**Last Updated:** 2026-02-06  
**Category:** Security  
**Tags:** rust, security, code-review, vulnerabilities

## Overview

Common security issues found in Rust projects and how to identify and fix them during code review.

## Credential Exposure

### Issue
Logging or displaying URLs that may contain embedded credentials.

### Detection
```rust
// ❌ Vulnerable
println!("Cloning from: {}", url);  // url might be "https://user:pass@github.com/repo"
Err(anyhow!("Failed to parse: {}", url))
```

### Fix
```rust
// ✅ Secure
fn sanitize_url(url: &str) -> String {
    // Remove everything before @ (credentials)
    url.split('@').last().unwrap_or(url).to_string()
}

println!("Cloning from: {}", sanitize_url(&url));
Err(anyhow!("Failed to parse: {}", sanitize_url(&url)))
```

### Testing
```rust
#[test]
fn test_sanitize_url() {
    assert_eq!(
        sanitize_url("https://user:pass@github.com/repo"),
        "github.com/repo"
    );
    assert_eq!(
        sanitize_url("https://github.com/repo"),
        "https://github.com/repo"
    );
}
```

## Token/Secret Handling in CI

### Issue
Piping secrets via stdin can expose them in logs if the command fails.

### Detection
```yaml
# ❌ Vulnerable
- run: echo "${{ secrets.TOKEN }}" | gh auth login --with-token
```

### Fix
```yaml
# ✅ Secure - Use environment variables
- run: gh pr view 1
  env:
    GH_TOKEN: ${{ secrets.TOKEN }}
```

### Rationale
- GitHub CLI automatically uses `GH_TOKEN` environment variable
- Environment variables are not shown in logs
- Stdin piping can be visible if the command errors

## Auto-merge Security

### Issue
Automatically merging PRs without manual review bypasses security checks.

### Detection
```yaml
# ❌ Risky
- name: Auto-approve
  run: gh pr review --approve
- name: Auto-merge
  run: gh pr merge --auto --squash
```

### Fix
```yaml
# ✅ Secure - Manual review required
# Remove auto-approve and auto-merge steps
# Document review requirements in CONTRIBUTING.md
```

### Rationale
- Humans should review security-sensitive changes
- Automated tests may not catch all vulnerabilities
- Manual review provides defense in depth

## Path Traversal

### Issue
User-controlled paths can escape intended directories.

### Detection
```rust
// ❌ Vulnerable
fn build_path(repo: &str) -> PathBuf {
    PathBuf::from(format!("/repos/{}", repo))  // repo could be "../../../etc/passwd"
}
```

### Fix
```rust
// ✅ Secure
fn build_path(repo: &str) -> Result<PathBuf> {
    let base = PathBuf::from("/repos");
    let full = base.join(repo);
    
    // Verify the path stays within base directory
    if !full.starts_with(&base) {
        return Err(anyhow!("Invalid repository path"));
    }
    
    Ok(full)
}
```

### Testing
```rust
#[test]
fn test_path_traversal() {
    assert!(build_path("valid/repo").is_ok());
    assert!(build_path("../../../etc/passwd").is_err());
    assert!(build_path("../../parent").is_err());
}
```

## Command Injection

### Issue
Passing unsanitized user input to shell commands.

### Detection
```rust
// ❌ Vulnerable
use std::process::Command;
Command::new("sh")
    .arg("-c")
    .arg(format!("git clone {}", repo))  // repo could contain "; rm -rf /"
```

### Fix
```rust
// ✅ Secure - Use command arguments, not shell
Command::new("git")
    .arg("clone")
    .arg(repo)  // Arguments are automatically escaped
    .output()?
```

### Rationale
- Direct command execution doesn't invoke shell
- Arguments are passed as arrays, not strings
- No shell interpretation of special characters

## Unsafe Code Review

### Issue
Unsafe blocks bypass Rust's memory safety guarantees.

### Detection
```rust
// 🔍 Review carefully
unsafe {
    // Any code here needs extra scrutiny
}
```

### Checklist
- [ ] Is `unsafe` necessary? Can it be avoided?
- [ ] Are all invariants documented?
- [ ] Are pointer dereferences checked for null?
- [ ] Are lifetimes and ownership respected?
- [ ] Is memory properly aligned?
- [ ] Are there tests for edge cases?

### Best Practice
```rust
// ✅ Document invariants
/// # Safety
/// 
/// This function is safe to call when:
/// - `ptr` is non-null
/// - `ptr` is properly aligned
/// - `ptr` points to valid data
unsafe fn read_ptr(ptr: *const u8) -> u8 {
    debug_assert!(!ptr.is_null());
    *ptr
}
```

## Dependency Security

### Issue
Outdated or vulnerable dependencies.

### Detection
```bash
cargo audit  # Check for known vulnerabilities
```

### Fix
```bash
cargo update  # Update dependencies
cargo audit fix  # Auto-fix vulnerabilities if possible
```

### Prevention
```yaml
# Add to CI workflow
- name: Security audit
  run: |
    cargo install cargo-audit
    cargo audit
```

## Error Handling Information Disclosure

### Issue
Error messages revealing internal implementation details.

### Detection
```rust
// ❌ Too detailed
Err(anyhow!("Database connection failed: postgres://user:pass@host:5432/db"))
Err(anyhow!("File not found: /home/user/.ssh/id_rsa"))
```

### Fix
```rust
// ✅ Generic but helpful
Err(anyhow!("Database connection failed"))
Err(anyhow!("Configuration file not found"))

// ✅ Log details internally, return generic message
log::error!("DB connection failed: {}", detailed_error);
Err(anyhow!("Database connection failed"))
```

## Input Validation

### Issue
Accepting untrusted input without validation.

### Detection
```rust
// ❌ No validation
fn process_url(url: String) -> Result<()> {
    // Directly uses url without checking
}
```

### Fix
```rust
// ✅ Validate input
use url::Url;

fn process_url(url: &str) -> Result<()> {
    // Parse and validate URL format
    let parsed = Url::parse(url)
        .map_err(|_| anyhow!("Invalid URL format"))?;
    
    // Validate scheme
    if !["http", "https", "git", "ssh"].contains(&parsed.scheme()) {
        return Err(anyhow!("Unsupported URL scheme"));
    }
    
    // Validate host exists
    if parsed.host_str().is_none() {
        return Err(anyhow!("URL must contain a host"));
    }
    
    Ok(())
}
```

## Regular Expression Denial of Service (ReDoS)

### Issue
Complex regex patterns with catastrophic backtracking.

### Detection
```rust
// ❌ Potentially vulnerable
let re = Regex::new(r"(a+)+b")?;  // Exponential time on input like "aaaaaa!"
```

### Fix
```rust
// ✅ Use simpler patterns or timeout
let re = Regex::new(r"a+b")?;  // Linear time

// Or use regex with timeout (regex crate doesn't support this natively)
// Consider using regex-automata or similar for complex patterns
```

## Security Review Checklist

When reviewing Rust code for security:

### Data Flow
- [ ] Identify all sources of untrusted input
- [ ] Trace data flow from input to sensitive operations
- [ ] Verify validation at trust boundaries
- [ ] Check for data sanitization before logging

### Authentication & Authorization
- [ ] Verify credentials are not logged or displayed
- [ ] Check for proper token/secret handling
- [ ] Ensure sensitive operations require authentication
- [ ] Verify authorization checks before privileged operations

### Error Handling
- [ ] Errors don't reveal sensitive information
- [ ] Errors are logged with appropriate detail
- [ ] User-facing errors are generic
- [ ] Panics are avoided in production code

### Dependencies
- [ ] Run `cargo audit` for known vulnerabilities
- [ ] Check dependency versions are up-to-date
- [ ] Review dependencies for trustworthiness
- [ ] Minimize dependency count

### Unsafe Code
- [ ] Justify necessity of `unsafe` blocks
- [ ] Document safety invariants
- [ ] Add tests for edge cases
- [ ] Consider safe alternatives

### External Commands
- [ ] Use direct execution, not shell commands
- [ ] Pass arguments as arrays, not strings
- [ ] Validate/sanitize command arguments
- [ ] Handle command failures gracefully

## Tools

### Static Analysis
```bash
cargo clippy -- -D warnings       # Linting
cargo audit                        # Vulnerability scanning
cargo deny check                   # License and security policy
cargo geiger                       # Unsafe code usage
```

### Dynamic Analysis
```bash
cargo test                         # Unit tests
cargo miri test                    # Undefined behavior detection
cargo fuzz                         # Fuzzing (requires setup)
```

## Resources

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Cargo Audit](https://github.com/RustSec/rustsec)
- [RustSec Advisory Database](https://rustsec.org/)
