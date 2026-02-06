use anyhow::{anyhow, Result};
use url::Url;

/// Sanitize a URL for error messages by removing any embedded credentials
fn sanitize_url(repo: &str) -> String {
    // Try to parse as URL and remove credentials
    if let Ok(mut url) = Url::parse(repo) {
        if url.username() != "" || url.password().is_some() {
            let _ = url.set_username("");
            let _ = url.set_password(None);
            return url.to_string();
        }
    }
    // If not a parseable URL or no credentials, return as-is
    repo.to_string()
}

/// Helper function to extract the full host from a repository URL.
fn extract_full_host(repo: &str) -> Result<String> {
    let trimmed = if let Some(stripped) = repo.strip_prefix("git+") {
        stripped
    } else {
        repo
    };

    // Try parsing as a URL (https, ssh://, etc.)
    if let Ok(url) = Url::parse(trimmed) {
        if let Some(host) = url.host_str() {
            return Ok(host.to_string());
        }
    }

    // Fallback: scp-like syntax git@host:owner/repo.git
    if trimmed.starts_with("git@") {
        if let Some((left, _right)) = trimmed.split_once(":") {
            if let Some((_, host)) = left.split_once("@") {
                return Ok(host.to_string());
            }
        }
    }

    // Handle plain host/path like "github.com/owner/repo.git"
    if let Some(first_segment) = trimmed.split('/').next() {
        if first_segment.contains('.') {
            return Ok(first_segment.to_string());
        }
    }

    Err(anyhow!(
        "Invalid Git repository URL: {}",
        sanitize_url(repo)
    ))
}

/// Return the repository host short label (e.g. "github") for a variety of
/// Git URL formats. This strips common TLDs so the local path uses the short
/// host name rather than the full domain.
pub fn get_host_from_repo(repo: &str) -> Result<String> {
    let full_host = extract_full_host(repo)?;
    let first = full_host
        .split('.')
        .next()
        .ok_or_else(|| anyhow!("Invalid host"))?;
    Ok(first.to_string())
}

/// Return the full host domain (e.g. "github.com") for callers that need it.
pub fn get_host_from_repo_full(repo: &str) -> Result<String> {
    extract_full_host(repo)
}

#[cfg(test)]
mod tests {
    use super::{get_host_from_repo, sanitize_url};

    #[test]
    fn test_sanitize_url_with_credentials() {
        let repo = "https://user:pass@github.com/owner/repo.git";
        let sanitized = sanitize_url(repo);
        assert!(!sanitized.contains("user"));
        assert!(!sanitized.contains("pass"));
        assert!(sanitized.contains("github.com"));
    }

    #[test]
    fn test_sanitize_url_without_credentials() {
        let repo = "https://github.com/owner/repo.git";
        let sanitized = sanitize_url(repo);
        assert_eq!(sanitized, repo);
    }

    #[test]
    fn test_https() {
        let repo = "https://github.com/owner/repo.git";
        let h = get_host_from_repo(repo).expect("should parse");
        assert_eq!(h, "github");
    }

    #[test]
    fn test_git_plus_https() {
        let repo = "git+https://gitlab.com/owner/repo.git";
        let h = get_host_from_repo(repo).expect("should parse");
        assert_eq!(h, "gitlab");
    }

    #[test]
    fn test_scp_style() {
        let repo = "git@bitbucket.org:owner/repo.git";
        let h = get_host_from_repo(repo).expect("should parse");
        assert_eq!(h, "bitbucket");
    }

    #[test]
    fn test_plain_host_path() {
        let repo = "github.com/owner/repo.git";
        let h = get_host_from_repo(repo).expect("should parse");
        assert_eq!(h, "github");
    }

    #[test]
    fn test_invalid() {
        let repo = "not-a-repo";
        assert!(get_host_from_repo(repo).is_err());
    }

    #[test]
    fn test_get_host_from_repo_full_https() {
        let repo = "https://github.com/owner/repo.git";
        let h = super::get_host_from_repo_full(repo).expect("should parse");
        assert_eq!(h, "github.com");
    }

    #[test]
    fn test_get_host_from_repo_full_scp() {
        let repo = "git@gitlab.com:owner/repo.git";
        let h = super::get_host_from_repo_full(repo).expect("should parse");
        assert_eq!(h, "gitlab.com");
    }

    #[test]
    fn test_get_host_from_repo_full_subdomain() {
        let repo = "https://git.example.com/owner/repo.git";
        let h = super::get_host_from_repo_full(repo).expect("should parse");
        assert_eq!(h, "git.example.com");
    }
}
