use anyhow::{anyhow, Result};
use url::Url;

/// Return the repository host short label (e.g. "github") for a variety of
/// Git URL formats. This strips common TLDs so the local path uses the short
/// host name rather than the full domain.
pub fn get_host_from_repo(repo: &str) -> Result<String> {
    // Short host (first segment) - existing default behavior
    // Normalize git+ prefix
    let trimmed = if let Some(stripped) = repo.strip_prefix("git+") {
        stripped
    } else {
        repo
    };

    // Try parsing as a URL (https, ssh://, etc.)
    if let Ok(url) = Url::parse(trimmed) {
        if let Some(host) = url.host_str() {
            let first = host
                .split('.')
                .next()
                .ok_or_else(|| anyhow!("Invalid host"))?;
            return Ok(first.to_string());
        }
    }

    // Fallback: scp-like syntax git@host:owner/repo.git
    if trimmed.starts_with("git@") {
        if let Some((left, _right)) = trimmed.split_once(":") {
            if let Some((_, host)) = left.split_once("@") {
                let first = host
                    .split('.')
                    .next()
                    .ok_or_else(|| anyhow!("Invalid host"))?;
                return Ok(first.to_string());
            }
        }
    }

    // Handle plain host/path like "github.com/owner/repo.git"
    if let Some(first_segment) = trimmed.split('/').next() {
        if first_segment.contains('.') {
            let first = first_segment
                .split('.')
                .next()
                .ok_or_else(|| anyhow!("Invalid host"))?;
            return Ok(first.to_string());
        }
    }

    Err(anyhow!("Invalid Git repository URL: {}", repo))
}

/// Return the full host domain (e.g. "github.com") for callers that need it.
pub fn get_host_from_repo_full(repo: &str) -> Result<String> {
    let trimmed = if let Some(stripped) = repo.strip_prefix("git+") {
        stripped
    } else {
        repo
    };

    if let Ok(url) = Url::parse(trimmed) {
        if let Some(host) = url.host_str() {
            return Ok(host.to_string());
        }
    }

    if trimmed.starts_with("git@") {
        if let Some((left, _right)) = trimmed.split_once(":") {
            if let Some((_, host)) = left.split_once("@") {
                return Ok(host.to_string());
            }
        }
    }

    if let Some(first_segment) = trimmed.split('/').next() {
        if first_segment.contains('.') {
            return Ok(first_segment.to_string());
        }
    }

    Err(anyhow!("Invalid Git repository URL: {}", repo))
}

#[cfg(test)]
mod tests {
    use super::get_host_from_repo;

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
}
