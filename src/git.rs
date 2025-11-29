use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use which::which;

pub fn clone_repo(repo: &str, local_path: &Path, dry_run: bool) -> Result<()> {
    let git = which("git").context("git executable not found in PATH")?;

    if dry_run {
        println!("Dry run: git clone {} {}", repo, local_path.display());
        return Ok(());
    }

    let status = Command::new(git)
        .arg("clone")
        .arg(repo)
        .arg(local_path)
        .status()
        .context("failed to spawn git clone")?;

    if !status.success() {
        anyhow::bail!("git clone failed with status: {}", status);
    }

    println!("Repository cloned successfully to {}", local_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::clone_repo;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_clone_repo_dry_run() {
        let repo = "git@github.com:owner/repo.git";
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_test_dry_run");

        // Ensure function returns Ok without performing network operations
        let res = clone_repo(repo, &tmp, true);
        assert!(res.is_ok());
    }
}
