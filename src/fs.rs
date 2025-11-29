use anyhow::Result;
use std::path::{PathBuf};
use dirs::home_dir;

/// Build the local path for a repo given root, host and repo path
/// Example: root=~/Projects, repo=git@github.com:owner/repo.git -> ~/Projects/github/owner/repo
pub fn build_local_repo_path(root: &str, repo: &str) -> Result<PathBuf> {
    // Resolve ~ in root
    let root_path = if root.starts_with("~") {
        let mut p = home_dir().ok_or_else(|| anyhow::anyhow!("Couldn't find home directory"))?;
        let suffix = root.trim_start_matches('~');
        if !suffix.is_empty() {
            p.push(&suffix[1..]);
        }
        p
    } else {
        PathBuf::from(root)
    };

    // Extract host and owner/repo
    let host = crate::util::get_host_from_repo(repo)?;

    // Extract owner/repo part
    let path_part = if let Some(_idx) = repo.find("/") {
        // https://host/owner/repo.git -> take after host
        let parts: Vec<&str> = repo.split('/').collect();
        // owner is at index 3 for https://host/owner/repo.git
        if parts.len() >= 4 {
            format!("{}/{}", parts[3], parts[4].trim_end_matches(".git"))
        } else {
            String::new()
        }
    } else if let Some(idx) = repo.find(":") {
        // git@host:owner/repo.git -> take after ':'
        let after = &repo[idx + 1..];
        let parts: Vec<&str> = after.split('/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1].trim_end_matches(".git"))
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
        let got = build_local_repo_path(root, repo).expect("build path");
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
        let got = build_local_repo_path(root, repo).expect("build path");
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
        let got = build_local_repo_path(root, repo).expect("build path");
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
        let got = build_local_repo_path(root, repo).expect("build path");
        let mut expected = PathBuf::from("/tmp/work");
        expected.push("github");
        expected.push("owner");
        expected.push("repo");
        assert_eq!(got, expected);
    }
}
