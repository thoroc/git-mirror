use std::fmt;

use url::Url;

/// A parsed git remote: host plus repository path (`namespace/repo`), without
/// the trailing `.git` and without any leading slash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub host: String,
    pub path: String,
}

impl Remote {
    /// Canonical identity used to compare remotes across URL forms
    /// (`git@host:ns/repo`, `ssh://git@host/ns/repo`, `https://...`).
    pub fn key(&self) -> String {
        format!("{}/{}", self.host.to_ascii_lowercase(), self.path)
    }
}

/// Why a remote string could not be turned into a `Remote`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteError {
    /// A local path or `file://` URL: valid git, but not a network host we
    /// can map into the projects tree.
    LocalPath(String),
    /// A network-looking string that is not a usable git remote.
    Invalid(String),
}

impl fmt::Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteError::LocalPath(url) => write!(f, "not a network git remote: {url}"),
            RemoteError::Invalid(url) => write!(f, "invalid git repository URL: {url}"),
        }
    }
}

impl std::error::Error for RemoteError {}

/// Sanitize a URL for error messages by removing any embedded credentials.
fn sanitize_url(repo: &str) -> String {
    if let Ok(mut url) = Url::parse(repo) {
        if url.username() != "" || url.password().is_some() {
            let _ = url.set_username("");
            let _ = url.set_password(None);
            return url.to_string();
        }
    }
    repo.to_string()
}

fn is_local_path(repo: &str) -> bool {
    repo.starts_with("file://")
        || repo.starts_with('/')
        || repo.starts_with("./")
        || repo.starts_with("../")
        || repo.starts_with('~')
        || (repo.contains('\\') && !repo.contains('@'))
}

/// Turn the path portion of a remote into `namespace/repo` form.
fn normalize_path(raw: &str) -> Result<String, RemoteError> {
    let mut segments = Vec::new();
    for segment in raw.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err(RemoteError::Invalid(
                "repository path must not contain '..'".to_string(),
            ));
        }
        segments.push(segment);
    }

    let mut path = segments.join("/");
    if let Some(stripped) = path.strip_suffix(".git") {
        path = stripped.to_string();
    }

    if path.is_empty() {
        return Err(RemoteError::Invalid(
            "repository URL has no path".to_string(),
        ));
    }

    Ok(path)
}

/// Parse the common git remote spellings into a host and repository path.
///
/// Supported forms:
/// - `git@github.com:owner/repo.git` (scp-like)
/// - `ssh://git@github.com/owner/repo.git`
/// - `https://github.com/owner/repo.git` (credentials and ports allowed)
/// - `git://github.com/owner/repo.git`
/// - `git+https://github.com/owner/repo.git`
/// - `github.com/owner/repo.git` (plain host/path)
pub fn parse_remote(repo: &str) -> Result<Remote, RemoteError> {
    let repo = repo.trim();
    if repo.is_empty() {
        return Err(RemoteError::Invalid("empty repository URL".to_string()));
    }

    // Some tools prefix the transport with `git+`.
    let repo = repo.strip_prefix("git+").unwrap_or(repo);

    if is_local_path(repo) {
        return Err(RemoteError::LocalPath(sanitize_url(repo)));
    }

    let (host, raw_path) = if repo.contains("://") {
        let url = Url::parse(repo).map_err(|e| RemoteError::Invalid(e.to_string()))?;
        let host = url
            .host_str()
            .ok_or_else(|| RemoteError::Invalid("repository URL has no host".to_string()))?;
        (host.to_string(), url.path().to_string())
    } else if let Some((left, right)) = repo.split_once(':') {
        // scp-like syntax: [user@]host:path
        let host = left.rsplit('@').next().unwrap_or(left);
        (host.to_string(), right.to_string())
    } else if let Some((host, path)) = repo.split_once('/') {
        // plain host/path, e.g. github.com/owner/repo.git
        if !host.contains('.') {
            return Err(RemoteError::Invalid(sanitize_url(repo)));
        }
        (host.to_string(), path.to_string())
    } else {
        return Err(RemoteError::Invalid(sanitize_url(repo)));
    };

    let host = host.to_ascii_lowercase();
    if host.is_empty() {
        return Err(RemoteError::Invalid(
            "repository URL has no host".to_string(),
        ));
    }

    Ok(Remote {
        host,
        path: normalize_path(&raw_path)?,
    })
}

/// Short local directory label for a host.
///
/// Known forges (and their self-hosted variants) get a stable short name so a
/// clone and a migration always agree. Unknown hosts fall back to their first
/// DNS label, which preserves the historical `git-mirror` behaviour.
pub fn host_label(host: &str) -> String {
    let host = host.to_ascii_lowercase();
    match host.as_str() {
        "github.com" => "github".to_string(),
        "gitlab.com" => "gitlab".to_string(),
        "bitbucket.org" => "bitbucket".to_string(),
        "go.googlesource.com" => "googlesource".to_string(),
        _ if host.contains("bitbucket") => "bitbucket".to_string(),
        _ if host.contains("gitlab") => "gitlab".to_string(),
        _ if host.contains("github") => "github".to_string(),
        _ if host.starts_with("phabricator.") => "phabricator".to_string(),
        _ => host.split('.').next().unwrap_or(&host).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{host_label, parse_remote, sanitize_url, RemoteError};

    fn remote(host: &str, path: &str) -> super::Remote {
        super::Remote {
            host: host.to_string(),
            path: path.to_string(),
        }
    }

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
        assert_eq!(sanitize_url(repo), repo);
    }

    #[test]
    fn test_parse_https() {
        let r = parse_remote("https://github.com/owner/repo.git").expect("parse");
        assert_eq!(r, remote("github.com", "owner/repo"));
    }

    #[test]
    fn test_parse_https_with_credentials() {
        let r = parse_remote("https://user@github.com/owner/repo.git").expect("parse");
        assert_eq!(r, remote("github.com", "owner/repo"));
    }

    #[test]
    fn test_parse_ssh_url() {
        let r = parse_remote("ssh://git@github.com/owner/repo.git").expect("parse");
        assert_eq!(r, remote("github.com", "owner/repo"));
    }

    #[test]
    fn test_parse_ssh_url_with_port() {
        let r = parse_remote("ssh://git@bitbucket.uk.novamedia.com:2222/awstl/ppl-cdk.git")
            .expect("parse");
        assert_eq!(r, remote("bitbucket.uk.novamedia.com", "awstl/ppl-cdk"));
    }

    #[test]
    fn test_parse_git_plus_https() {
        let r = parse_remote("git+https://gitlab.com/owner/repo.git").expect("parse");
        assert_eq!(r, remote("gitlab.com", "owner/repo"));
    }

    #[test]
    fn test_parse_git_protocol() {
        let r = parse_remote("git://github.com/owner/repo.git").expect("parse");
        assert_eq!(r, remote("github.com", "owner/repo"));
    }

    #[test]
    fn test_parse_scp_style() {
        let r = parse_remote("git@bitbucket.org:owner/repo.git").expect("parse");
        assert_eq!(r, remote("bitbucket.org", "owner/repo"));
    }

    #[test]
    fn test_parse_scp_style_double_slash() {
        let r = parse_remote("git@bitbucket.org:pplcs//php-monolith.git").expect("parse");
        assert_eq!(r, remote("bitbucket.org", "pplcs/php-monolith"));
    }

    #[test]
    fn test_parse_plain_host_path() {
        let r = parse_remote("github.com/owner/repo.git").expect("parse");
        assert_eq!(r, remote("github.com", "owner/repo"));
    }

    #[test]
    fn test_parse_subgroups_are_preserved() {
        let r = parse_remote("git@gitlab.com:group/subgroup/project.git").expect("parse");
        assert_eq!(r, remote("gitlab.com", "group/subgroup/project"));
    }

    #[test]
    fn test_parse_trailing_slash() {
        let r = parse_remote("https://github.com/owner/repo.git/").expect("parse");
        assert_eq!(r, remote("github.com", "owner/repo"));
    }

    #[test]
    fn test_parse_local_path_is_rejected() {
        assert!(matches!(
            parse_remote("/tmp/owner/repo"),
            Err(RemoteError::LocalPath(_))
        ));
        assert!(matches!(
            parse_remote("file:///var/jenkins_workspace/my-pipeline"),
            Err(RemoteError::LocalPath(_))
        ));
        assert!(matches!(
            parse_remote("./owner/repo"),
            Err(RemoteError::LocalPath(_))
        ));
    }

    #[test]
    fn test_parse_invalid() {
        assert!(matches!(
            parse_remote("not-a-repo"),
            Err(RemoteError::Invalid(_))
        ));
        assert!(matches!(
            parse_remote("https://github.com/"),
            Err(RemoteError::Invalid(_))
        ));
    }

    #[test]
    fn test_host_label_known_forges() {
        assert_eq!(host_label("github.com"), "github");
        assert_eq!(host_label("gitlab.com"), "gitlab");
        assert_eq!(host_label("bitbucket.org"), "bitbucket");
    }

    #[test]
    fn test_host_label_self_hosted_forges() {
        assert_eq!(host_label("bitbucket.uk.novamedia.com"), "bitbucket");
        assert_eq!(host_label("gitlab.example.com"), "gitlab");
        assert_eq!(host_label("github.example.com"), "github");
    }

    #[test]
    fn test_host_label_other_hosts() {
        assert_eq!(host_label("go.googlesource.com"), "googlesource");
        assert_eq!(host_label("gopkg.in"), "gopkg");
        assert_eq!(host_label("phabricator.mousepawmedia.net"), "phabricator");
        assert_eq!(host_label("git.example.com"), "git");
    }

    fn label(repo: &str) -> String {
        host_label(&parse_remote(repo).expect("parse").host)
    }

    #[test]
    fn test_label_https() {
        assert_eq!(label("https://github.com/owner/repo.git"), "github");
    }

    #[test]
    fn test_label_git_plus_https() {
        assert_eq!(label("git+https://gitlab.com/owner/repo.git"), "gitlab");
    }

    #[test]
    fn test_label_scp_style() {
        assert_eq!(label("git@bitbucket.org:owner/repo.git"), "bitbucket");
    }

    #[test]
    fn test_label_plain_host_path() {
        assert_eq!(label("github.com/owner/repo.git"), "github");
    }

    #[test]
    fn test_label_googlesource() {
        assert_eq!(label("https://go.googlesource.com/crypto"), "googlesource");
    }

    #[test]
    fn test_label_invalid() {
        assert!(parse_remote("not-a-repo").is_err());
    }

    #[test]
    fn test_full_host_https() {
        assert_eq!(
            parse_remote("https://github.com/owner/repo.git")
                .expect("parse")
                .host,
            "github.com"
        );
    }

    #[test]
    fn test_full_host_scp() {
        assert_eq!(
            parse_remote("git@gitlab.com:owner/repo.git")
                .expect("parse")
                .host,
            "gitlab.com"
        );
    }

    #[test]
    fn test_full_host_subdomain() {
        assert_eq!(
            parse_remote("https://git.example.com/owner/repo.git")
                .expect("parse")
                .host,
            "git.example.com"
        );
    }

    #[test]
    fn test_remote_key_matches_url_forms() {
        let scp = parse_remote("git@bitbucket.org:pplcs/repo.git").expect("parse");
        let https =
            parse_remote("https://thomasaroche@bitbucket.org/pplcs/repo.git").expect("parse");
        assert_eq!(scp.key(), https.key());
    }
}
