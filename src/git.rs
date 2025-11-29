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
