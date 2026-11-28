use anyhow::Result;
use dirs::home_dir;
use std::path::{Path, PathBuf};

use crate::util::{host_label, parse_remote, Remote};

/// Resolve a `~`-prefixed root into an absolute path.
pub fn expand_root(root: &str) -> Result<PathBuf> {
    let root_path = if let Some(suffix) = root.strip_prefix('~') {
        let mut p = home_dir().ok_or_else(|| anyhow::anyhow!("Couldn't find home directory"))?;
        let suffix = suffix.trim_start_matches('/');
        if !suffix.is_empty() {
            p.push(suffix);
        }
        p
    } else {
        PathBuf::from(root)
    };
    Ok(root_path)
}

/// Build the local path for an already-parsed remote.
///
/// Example: root=~/Projects, remote=github.com/owner/repo
/// -> ~/Projects/github/owner/repo (or ~/Projects/github.com/owner/repo with
/// `full_host`).
pub fn target_path(root: &Path, remote: &Remote, full_host: bool) -> PathBuf {
    let host_dir = if full_host {
        remote.host.clone()
    } else {
        host_label(&remote.host)
    };

    let mut local = root.to_path_buf();
    local.push(host_dir);
    for segment in remote.path.split('/') {
        local.push(segment);
    }
    local
}

/// Build the local path for a repo given root, host and repo path.
///
/// Example: root=~/Projects, repo=git@github.com:owner/repo.git
/// -> ~/Projects/github/owner/repo
pub fn build_local_repo_path(root: &str, repo: &str, full_host: bool) -> Result<PathBuf> {
    let remote = parse_remote(repo)?;
    Ok(target_path(&expand_root(root)?, &remote, full_host))
}

/// Remove now-empty parent directories of `source`, stopping at `root`.
pub fn prune_empty_parents(source: &Path, root: &Path) {
    let mut dir = source.parent();

    while let Some(candidate) = dir {
        if candidate == root {
            break;
        }
        if std::fs::remove_dir(candidate).is_err() {
            break;
        }
        dir = candidate.parent();
    }
}

#[cfg(test)]
mod tests {
    use super::{build_local_repo_path, expand_root};
    use dirs::home_dir;
    use std::path::PathBuf;

    fn expected(segments: &[&str]) -> PathBuf {
        let mut p = home_dir().expect("home_dir");
        p.push("Projects");
        for segment in segments {
            p.push(segment);
        }
        p
    }

    #[test]
    fn test_https_path() {
        let got = build_local_repo_path("~/Projects", "https://github.com/owner/repo.git", false)
            .expect("build path");
        assert_eq!(got, expected(&["github", "owner", "repo"]));
    }

    #[test]
    fn test_https_with_credentials_path() {
        let got = build_local_repo_path(
            "~/Projects",
            "https://user@github.com/owner/repo.git",
            false,
        )
        .expect("build path");
        assert_eq!(got, expected(&["github", "owner", "repo"]));
    }

    #[test]
    fn test_ssh_url_path() {
        let got = build_local_repo_path("~/Projects", "ssh://git@github.com/owner/repo.git", false)
            .expect("build path");
        assert_eq!(got, expected(&["github", "owner", "repo"]));
    }

    #[test]
    fn test_ssh_url_with_port_path() {
        let got = build_local_repo_path(
            "~/Projects",
            "ssh://git@bitbucket.uk.novamedia.com:2222/awstl/ppl-cdk.git",
            false,
        )
        .expect("build path");
        assert_eq!(got, expected(&["bitbucket", "awstl", "ppl-cdk"]));
    }

    #[test]
    fn test_git_plus_https_path() {
        let got =
            build_local_repo_path("~/Projects", "git+https://gitlab.com/owner/repo.git", false)
                .expect("build path");
        assert_eq!(got, expected(&["gitlab", "owner", "repo"]));
    }

    #[test]
    fn test_scp_style_path() {
        let got = build_local_repo_path("~/Projects", "git@bitbucket.org:owner/repo.git", false)
            .expect("build path");
        assert_eq!(got, expected(&["bitbucket", "owner", "repo"]));
    }

    #[test]
    fn test_subgroup_path() {
        let got = build_local_repo_path(
            "~/Projects",
            "git@gitlab.com:group/subgroup/project.git",
            false,
        )
        .expect("build path");
        assert_eq!(got, expected(&["gitlab", "group", "subgroup", "project"]));
    }

    #[test]
    fn test_googlesource_path() {
        let got = build_local_repo_path("~/Projects", "https://go.googlesource.com/crypto", false)
            .expect("build path");
        assert_eq!(got, expected(&["googlesource", "crypto"]));
    }

    #[test]
    fn test_custom_root() {
        let got = build_local_repo_path("/tmp/work", "git@github.com:owner/repo.git", false)
            .expect("build path");
        let expected = PathBuf::from("/tmp/work").join("github/owner/repo");
        assert_eq!(got, expected);
    }

    #[test]
    fn test_full_host_true() {
        let got = build_local_repo_path("~/Projects", "git@github.com:owner/repo.git", true)
            .expect("build path");
        assert_eq!(got, expected(&["github.com", "owner", "repo"]));
    }

    #[test]
    fn test_expand_root_tilde() {
        let got = expand_root("~/Projects").expect("expand");
        let mut expected = home_dir().expect("home_dir");
        expected.push("Projects");
        assert_eq!(got, expected);
    }

    #[test]
    fn test_expand_root_plain() {
        assert_eq!(
            expand_root("/tmp/work").expect("expand"),
            PathBuf::from("/tmp/work")
        );
    }
}
