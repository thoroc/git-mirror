use anyhow::{anyhow, Result};
use url::Url;

pub fn get_host_from_repo(repo: &str) -> Result<String> {
    // Handle git+https:// prefix
    let trimmed = if let Some(stripped) = repo.strip_prefix("git+") {
        stripped
    } else {
        repo
    };

    // Try parsing as a URL (https)
    if let Ok(url) = Url::parse(trimmed) {
        if let Some(host) = url.host_str() {
            // Return first segment (e.g. "github" from "github.com")
            let first = host.split('.').next().ok_or_else(|| anyhow!("Invalid host"))?;
            return Ok(first.to_string());
        }
    }

    // Fallback: scp-like syntax git@host:owner/repo.git
    if trimmed.starts_with("git@") {
        // split at ':' then extract host from left part
        if let Some((left, _right)) = trimmed.split_once(":") {
            if let Some((_, host)) = left.split_once("@") {
                let first = host.split('.').next().ok_or_else(|| anyhow!("Invalid host"))?;
                return Ok(first.to_string());
            }
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
    fn test_invalid() {
        let repo = "not-a-repo";
        assert!(get_host_from_repo(repo).is_err());
    }
}
