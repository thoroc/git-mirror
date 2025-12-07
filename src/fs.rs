use anyhow::Result;
use dirs::home_dir;
use std::path::PathBuf;

/// Build the local path for a repo given root, host and repo path
/// Example: root=~/Projects, repo=git@github.com:owner/repo.git -> ~/Projects/github.com/owner/repo
pub fn build_local_repo_path(root: &str, repo: &str, full_host: bool) -> Result<PathBuf> {
    // Resolve ~ in root (handle "~/..." and "~" only)
    let root_path = if root.starts_with("~") {
        let mut p = home_dir().ok_or_else(|| anyhow::anyhow!("Couldn't find home directory"))?;
        let suffix = root.trim_start_matches('~');
        if !suffix.is_empty() {
            // suffix starts with '/', remove it before pushing
            let to_push = suffix.trim_start_matches('/');
            p.push(to_push);
        }
        p
    } else {
        PathBuf::from(root)
    };

    // Extract host and owner/repo
    let host = if full_host {
        crate::util::get_host_from_repo_full(repo)?
    } else {
        crate::util::get_host_from_repo(repo)?
    };

    // Extract path part preserving nested segments (groups/subgroups/repo)
    let path_part = if repo.contains(":") && repo.contains("@") {
        // scp-style: git@host:owner/subgroup/repo.git -> take after ':'
        let idx = repo.find(":").unwrap();
        let after = &repo[idx + 1..];
        // remove .git suffix if present
        let trimmed = after.trim_end_matches(".git");
        trimmed.to_string()
    } else if repo.contains("/") {
        // URL or plain host/path. Try to parse after host segment
        // For URLs like https://host/owner/sub/repo.git or plain host/path like host/owner/repo.git
        // Split on '/', drop leading scheme+host if present
        let parts: Vec<&str> = repo.split('/').collect();
        // If repo starts with a scheme like "https:", shift to get host at index 2
        if parts.len() >= 3 && parts[0].ends_with(":") {
            // e.g. ["https:", "", "github.com", "owner", "repo.git"]
            if parts.len() >= 4 {
                let relevant = &parts[3..];
                let joined = relevant.join("/").trim_end_matches(".git").to_string();
                joined
            } else {
                String::new()
            }
        } else if parts.len() >= 2 {
            // plain host/path like github.com/owner/repo.git -> drop the host
            let relevant = &parts[1..];
            let joined = relevant.join("/").trim_end_matches(".git").to_string();
            joined
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let mut local = root_path;
    local.push(host);
    if !path_part.is_empty() {
        for seg in path_part.split('/') {
            local.push(seg);
        }
    }

    Ok(local)
}

#[cfg(test)]
mod tests {
    use super::build_local_repo_path;
    use dirs::home_dir;
    use std::path::PathBuf;

    #[test]
    fn test_https_path() {
        let root = "~/Projects";
        let repo = "https://github.com/owner/repo.git";
        let got = build_local_repo_path(root, repo, false).expect("build path");
        let mut expected = home_dir().expect("home_dir");
        expected.push("Projects");
        expected.push("github");
        expected.push("owner");
        expected.push("repo");
        assert_eq!(got, expected);
    }

    #[test]
    fn test_git_plus_https_path() {
        let root = "~/Projects";
        let repo = "git+https://gitlab.com/owner/repo.git";
        let got = build_local_repo_path(root, repo, false).expect("build path");
        let mut expected = home_dir().expect("home_dir");
        expected.push("Projects");
        expected.push("gitlab");
        expected.push("owner");
        expected.push("repo");
        assert_eq!(got, expected);
    }

    #[test]
    fn test_scp_style_path() {
        let root = "~/Projects";
        let repo = "git@bitbucket.org:owner/repo.git";
        let got = build_local_repo_path(root, repo, false).expect("build path");
        let mut expected = home_dir().expect("home_dir");
        expected.push("Projects");
        expected.push("bitbucket");
        expected.push("owner");
        expected.push("repo");
        assert_eq!(got, expected);
    }

    #[test]
    fn test_custom_root() {
        let root = "/tmp/work";
        let repo = "git@github.com:owner/repo.git";
        let got = build_local_repo_path(root, repo, false).expect("build path");
        let mut expected = PathBuf::from("/tmp/work");
        expected.push("github");
        expected.push("owner");
        expected.push("repo");
        assert_eq!(got, expected);
    }

    #[test]
    fn test_full_host_true() {
        let root = "~/Projects";
        let repo = "git@github.com:owner/repo.git";
        let got = build_local_repo_path(root, repo, true).expect("build path");
        let mut expected = home_dir().expect("home_dir");
        expected.push("Projects");
        expected.push("github.com");
        expected.push("owner");
        expected.push("repo");
        assert_eq!(got, expected);
    }
}
