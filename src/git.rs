use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use which::which;
use std::io::Write;

fn clone_repo_with_writer<W: Write>(repo: &str, local_path: &Path, dry_run: bool, writer: &mut W) -> Result<()> {
    let git = which("git").context("git executable not found in PATH")?;

    if dry_run {
        writeln!(writer, "Dry run: git clone {} {}", repo, local_path.display())?;
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

    writeln!(writer, "Repository cloned successfully to {}", local_path.display())?;
    Ok(())
}

pub fn clone_repo(repo: &str, local_path: &Path, dry_run: bool) -> Result<()> {
    let mut stdout = std::io::stdout();
    clone_repo_with_writer(repo, local_path, dry_run, &mut stdout)
}

#[cfg(test)]
mod tests {
    use super::{clone_repo, clone_repo_with_writer};
    use std::env;

    #[test]
    fn test_clone_repo_dry_run() {
        let repo = "git@github.com:owner/repo.git";
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_test_dry_run");

        // Ensure function returns Ok without performing network operations
        let res = clone_repo(repo, &tmp, true);
        assert!(res.is_ok());
    }

    #[test]
    fn test_clone_repo_dry_run_captures_output() {
        let repo = "git@github.com:owner/repo.git";
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_test_dry_run_capture");

        let mut buf: Vec<u8> = Vec::new();
        let res = clone_repo_with_writer(repo, &tmp, true, &mut buf);
        assert!(res.is_ok());
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("Dry run: git clone"));
        assert!(s.contains(&tmp.display().to_string()));
    }
}
