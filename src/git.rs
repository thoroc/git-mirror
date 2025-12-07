use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use which::which;

fn clone_repo_with_writer<W: Write>(
    repo: &str,
    local_path: &Path,
    dry_run: bool,
    writer: &mut W,
) -> Result<()> {
    let git = which("git").context("git executable not found in PATH")?;

    if dry_run {
        writeln!(
            writer,
            "Dry run: git clone {} {}",
            repo,
            local_path.display()
        )?;
        return Ok(());
    }

    let parent = local_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid local path"))?;
    fs::create_dir_all(parent).context("failed to create parent directories")?;

    let status = Command::new(git)
        .arg("clone")
        .arg(repo)
        .arg(local_path)
        .status()
        .context("failed to spawn git clone")?;

    if !status.success() {
        anyhow::bail!("git clone failed with status: {}", status);
    }

    writeln!(
        writer,
        "Repository cloned successfully to {}",
        local_path.display()
    )?;
    Ok(())
}

pub fn clone_repo(repo: &str, local_path: &Path, dry_run: bool) -> Result<()> {
    let mut stdout = std::io::stdout();
    clone_repo_with_writer(repo, local_path, dry_run, &mut stdout)
}

/// Return true if the given local path appears to be an existing git repository
pub fn repo_exists(local_path: &Path) -> bool {
    local_path.exists() && local_path.join(".git").is_dir()
}

/// Build the commands to show when a repo already exists locally.
/// If `print_cd` is true, return only the `cd` command, otherwise return `cd` + `git -C ... pull --ff-only`.
#[cfg(test)]
fn existing_repo_commands(local_path: &Path, print_cd: bool) -> String {
    if print_cd {
        format!("cd \"{}\"", local_path.display())
    } else {
        let cd = format!("cd \"{}\"", local_path.display());
        let pull = format!("git -C \"{}\" pull --ff-only", local_path.display());
        format!("{}\n{}", cd, pull)
    }
}

/// Fetch an existing repository using `git -C <path> fetch --all --prune`
pub fn fetch_repo(local_path: &Path, dry_run: bool) -> Result<()> {
    let git = which("git").context("git executable not found in PATH")?;

    if dry_run {
        println!(
            "Dry run: git -C {} fetch --all --prune",
            local_path.display()
        );
        return Ok(());
    }

    let status = Command::new(git)
        .arg("-C")
        .arg(local_path)
        .arg("fetch")
        .arg("--all")
        .arg("--prune")
        .status()
        .context("failed to spawn git fetch")?;

    if !status.success() {
        anyhow::bail!("git fetch failed with status: {}", status);
    }

    println!("Fetched repository at {}", local_path.display());
    Ok(())
}

/// Open the given local path in VS Code using the `code` command-line tool.
/// If `dry_run` is true, print the command that would be executed instead of running it.
pub fn open_in_vscode(local_path: &Path, dry_run: bool) -> Result<()> {
    let mut stdout = std::io::stdout();
    open_in_vscode_with_writer(local_path, dry_run, &mut stdout)
}

fn open_in_vscode_with_writer<W: Write>(
    local_path: &Path,
    dry_run: bool,
    writer: &mut W,
) -> Result<()> {
    let code = which("code").context("`code` executable not found in PATH")?;

    if dry_run {
        writeln!(writer, "Dry run: open VS Code at {}", local_path.display())?;
        return Ok(());
    }

    let status = Command::new(code)
        .arg(local_path)
        .status()
        .context("failed to spawn code")?;

    if !status.success() {
        anyhow::bail!("`code` failed with status: {}", status);
    }

    writeln!(writer, "Opened VS Code at {}", local_path.display())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        clone_repo_with_writer, existing_repo_commands, fetch_repo, open_in_vscode_with_writer,
        repo_exists,
    };
    use std::env;
    use std::fs;

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

    #[test]
    fn test_repo_exists_false() {
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_repo_exists_false_unit");
        let _ = fs::remove_dir_all(&tmp);
        assert!(!repo_exists(&tmp));
    }

    #[test]
    fn test_repo_exists_true() {
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_repo_exists_true_unit");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join(".git")).expect("create .git");
        assert!(repo_exists(&tmp));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_existing_repo_commands_print_cd() {
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_existing_commands_cd_unit");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("create tmp");
        let s = existing_repo_commands(&tmp, true);
        assert!(s.starts_with("cd \""));
        assert!(!s.contains("pull"));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_existing_repo_commands_pull() {
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_existing_commands_pull_unit");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("create tmp");
        let s = existing_repo_commands(&tmp, false);
        assert!(s.contains("git -C"));
        assert!(s.contains("pull --ff-only"));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_open_in_vscode_dry_run() {
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_open_vscode_dry_run");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("create tmp");

        let mut buf: Vec<u8> = Vec::new();
        let res = open_in_vscode_with_writer(&tmp, true, &mut buf);
        assert!(res.is_ok());
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("Dry run: open VS Code"));
        assert!(s.contains(&tmp.display().to_string()));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_fetch_repo_dry_run() {
        let mut tmp = env::temp_dir();
        tmp.push("git_mirror_fetch_repo_dry_run");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("create tmp");

        let res = fetch_repo(&tmp, true);
        assert!(res.is_ok());
        let _ = fs::remove_dir_all(&tmp);
    }
}
