use std::collections::HashMap;
use std::fs as stdfs;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::fs;
use crate::git;
use crate::util::{self, RemoteError};

const RESET: &str = "\x1b[0m";
const GREEN: &str = "32";
const CYAN: &str = "36";
const YELLOW: &str = "33";
const RED: &str = "31";
const DIM: &str = "2";

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    Text,
    Tsv,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Format::Text => "text",
            Format::Tsv => "tsv",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Move,
    Moved,
    Ok,
    NoOrigin,
    Unsupported,
    Excluded,
    Duplicate,
    Collision,
    Conflict,
    Worktree,
    Unparsable,
    Root,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Move => "move",
            Status::Moved => "moved",
            Status::Ok => "ok",
            Status::NoOrigin => "no-origin",
            Status::Unsupported => "unsupported-remote",
            Status::Excluded => "excluded",
            Status::Duplicate => "duplicate",
            Status::Collision => "collision",
            Status::Conflict => "conflict",
            Status::Worktree => "worktree",
            Status::Unparsable => "unparsable",
            Status::Root => "root",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub status: Status,
    pub current: PathBuf,
    pub origin: Option<String>,
    pub target: Option<PathBuf>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub moved: usize,
    pub planned: usize,
    pub already: usize,
    pub no_origin: usize,
    pub unsupported: usize,
    pub excluded: usize,
    pub conflicts: usize,
    pub duplicates: usize,
    pub worktrees: usize,
    pub unparsable: usize,
    pub root: usize,
    pub failed: usize,
}

impl Summary {
    fn record(&mut self, status: Status) {
        match status {
            Status::Move => self.planned += 1,
            Status::Moved => self.moved += 1,
            Status::Ok => self.already += 1,
            Status::NoOrigin => self.no_origin += 1,
            Status::Unsupported => self.unsupported += 1,
            Status::Excluded => self.excluded += 1,
            Status::Duplicate => self.duplicates += 1,
            Status::Collision | Status::Conflict => self.conflicts += 1,
            Status::Worktree => self.worktrees += 1,
            Status::Unparsable => self.unparsable += 1,
            Status::Root => self.root += 1,
        }
    }
}

pub struct Options {
    pub root: PathBuf,
    pub apply: bool,
    pub include_nested: bool,
    pub force: bool,
    pub full_host: bool,
    pub excludes: Vec<PathBuf>,
    pub format: Format,
}

/// Classify every repository under the root, optionally moving the ones that
/// need it. Returns the summary; individual move failures are reported but do
/// not abort the run.
pub fn run(opts: &Options, out: &mut dyn Write, err: &mut dyn Write) -> Result<Summary> {
    let color = opts.format == Format::Text && std::io::stdout().is_terminal();
    let repos = find_repos(&opts.root, opts.include_nested)?;
    let mut summary = Summary::default();
    let mut claimed: HashMap<PathBuf, (String, PathBuf)> = HashMap::new();

    if opts.format == Format::Tsv {
        writeln!(out, "status\tcurrent\torigin\ttarget")?;
    }

    for current in repos {
        let mut entry = classify(&current, opts, &mut claimed)?;

        if entry.status == Status::Move && opts.apply {
            match apply_move(&entry, &opts.root) {
                Ok(()) => entry.status = Status::Moved,
                Err(error) => {
                    summary.failed += 1;
                    let target = entry
                        .target
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_default();
                    writeln!(
                        err,
                        "{} {} -> {}: {error:#}",
                        paint("FAIL", RED, color),
                        entry.current.display(),
                        target
                    )?;
                    continue;
                }
            }
        }

        summary.record(entry.status);
        emit(&entry, opts.format, color, out)?;
    }

    if opts.format == Format::Tsv {
        print_summary(&summary, opts.apply, err)?;
    } else {
        print_summary(&summary, opts.apply, out)?;
    }

    Ok(summary)
}

fn entry(status: Status, current: &Path) -> Entry {
    Entry {
        status,
        current: current.to_path_buf(),
        origin: None,
        target: None,
        detail: None,
    }
}

fn classify(
    current: &Path,
    opts: &Options,
    claimed: &mut HashMap<PathBuf, (String, PathBuf)>,
) -> Result<Entry> {
    if current == opts.root {
        return Ok(entry(Status::Root, current));
    }

    if is_excluded(current, &opts.excludes) {
        return Ok(entry(Status::Excluded, current));
    }

    let Some(origin) = git::origin_url(current)? else {
        return Ok(entry(Status::NoOrigin, current));
    };

    let remote = match util::parse_remote(&origin) {
        Ok(remote) => remote,
        Err(RemoteError::LocalPath(_)) => {
            let mut entry = entry(Status::Unsupported, current);
            entry.origin = Some(origin);
            return Ok(entry);
        }
        Err(RemoteError::Invalid(_)) => {
            let mut entry = entry(Status::Unparsable, current);
            entry.origin = Some(origin);
            return Ok(entry);
        }
    };

    let target = fs::target_path(&opts.root, &remote, opts.full_host);
    let key = remote.key();

    if let Some((claimed_key, claimed_source)) = claimed.get(&target) {
        let mut entry = entry(
            if *claimed_key == key {
                Status::Duplicate
            } else {
                Status::Collision
            },
            current,
        );
        entry.origin = Some(origin);
        entry.target = Some(target);
        entry.detail = Some(if *claimed_key == key {
            claimed_source.display().to_string()
        } else {
            format!("{claimed_key} vs {key}")
        });
        return Ok(entry);
    }

    claimed.insert(target.clone(), (key.clone(), current.to_path_buf()));

    if current == target {
        let mut entry = entry(Status::Ok, current);
        entry.origin = Some(origin);
        entry.target = Some(target);
        return Ok(entry);
    }

    if target.exists() || target.symlink_metadata().is_ok() {
        let same_remote = git::origin_url(&target)?
            .and_then(|url| util::parse_remote(&url).ok())
            .map(|existing| existing.key() == key)
            .unwrap_or(false);

        let mut entry = entry(Status::Conflict, current);
        entry.origin = Some(origin);
        entry.target = Some(target);
        entry.detail = Some(if same_remote {
            " with the same remote".to_string()
        } else {
            format!(" (from {})", current.display())
        });
        return Ok(entry);
    }

    if !opts.force && has_linked_worktrees(current) {
        let mut entry = entry(Status::Worktree, current);
        entry.origin = Some(origin);
        entry.target = Some(target);
        return Ok(entry);
    }

    let mut entry = entry(Status::Move, current);
    entry.origin = Some(origin);
    entry.target = Some(target);
    Ok(entry)
}

fn is_excluded(path: &Path, excludes: &[PathBuf]) -> bool {
    excludes
        .iter()
        .any(|prefix| path == prefix || path.starts_with(prefix))
}

fn has_linked_worktrees(repo: &Path) -> bool {
    match stdfs::read_dir(repo.join(".git").join("worktrees")) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => false,
    }
}

fn find_repos(root: &Path, include_nested: bool) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();

    if root.join(".git").is_dir() {
        repos.push(root.to_path_buf());
    }
    walk(root, &mut repos, include_nested);

    // Deepest paths first so nested repositories move before their parents.
    repos.sort_by(|a, b| {
        depth(b)
            .cmp(&depth(a))
            .then_with(|| a.as_os_str().cmp(b.as_os_str()))
    });

    Ok(repos)
}

fn depth(path: &Path) -> usize {
    path.components().count()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, descend_into_repos: bool) {
    let entries = match stdfs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if !file_type.is_dir() {
            continue;
        }

        if entry.file_name().to_string_lossy() == ".git" {
            continue;
        }

        let path = entry.path();
        let is_repo = path.join(".git").is_dir();
        if is_repo {
            out.push(path.clone());
            // Nested repositories are skipped by default, so there is no
            // reason to walk the (often huge) working tree of a repo.
            if !descend_into_repos {
                continue;
            }
        }
        walk(&path, out, descend_into_repos);
    }
}

fn apply_move(entry: &Entry, root: &Path) -> Result<()> {
    let target = entry
        .target
        .as_ref()
        .ok_or_else(|| anyhow!("move entry has no target"))?;

    if let Some(parent) = target.parent() {
        stdfs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }

    stdfs::rename(&entry.current, target).with_context(|| {
        format!(
            "cannot move {} to {}",
            entry.current.display(),
            target.display()
        )
    })?;

    prune_empty_parents(&entry.current, root);
    Ok(())
}

fn prune_empty_parents(source: &Path, root: &Path) {
    let mut dir = source.parent();
    while let Some(candidate) = dir {
        if candidate == root {
            break;
        }
        if stdfs::remove_dir(candidate).is_err() {
            break;
        }
        dir = candidate.parent();
    }
}

fn paint(text: &str, code: &str, enabled: bool) -> String {
    if enabled {
        format!("\x1b[{code}m{text}{RESET}")
    } else {
        text.to_string()
    }
}

fn emit(entry: &Entry, format: Format, color: bool, out: &mut dyn Write) -> Result<()> {
    if format == Format::Tsv {
        writeln!(
            out,
            "{}\t{}\t{}\t{}",
            entry.status.as_str(),
            entry.current.display(),
            entry.origin.as_deref().unwrap_or(""),
            entry
                .target
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default()
        )?;
    } else {
        writeln!(out, "{}", text_line(entry, color))?;
    }
    Ok(())
}

fn text_line(entry: &Entry, color: bool) -> String {
    let current = entry.current.display().to_string();
    let target = entry
        .target
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let origin = entry.origin.as_deref().unwrap_or("");
    let detail = entry.detail.as_deref().unwrap_or("");
    let skip = paint("SKIP", YELLOW, color);

    match entry.status {
        Status::Move => format!("{} {current} -> {target}", paint("PLAN", CYAN, color)),
        Status::Moved => format!("{} {current} -> {target}", paint("MOVE", GREEN, color)),
        Status::Ok => paint(&format!("  ok: {current}"), DIM, color),
        Status::Root => paint(&format!("  skip projects root repo: {current}"), DIM, color),
        Status::NoOrigin => format!("{skip} no origin remote: {current}"),
        Status::Unsupported => format!("{skip} unsupported remote ({origin}): {current}"),
        Status::Excluded => format!("{skip} excluded: {current}"),
        Status::Duplicate => {
            format!("{skip} duplicate checkout of the same remote (keeping {detail}): {current}")
        }
        Status::Collision => {
            format!("{skip} target collision, different remotes ({detail}): {target}")
        }
        Status::Conflict => format!("{skip} target already exists{detail}: {target}"),
        Status::Worktree => format!("{skip} linked worktrees present: {current}"),
        Status::Unparsable => format!("{skip} unparsable origin ({origin}): {current}"),
    }
}

fn print_summary(summary: &Summary, apply: bool, out: &mut dyn Write) -> Result<()> {
    writeln!(out)?;

    if apply {
        writeln!(out, "Moved: {}", summary.moved)?;
    } else {
        writeln!(
            out,
            "Dry run: {} repo(s) would move. Re-run with --apply to perform the moves.",
            summary.planned
        )?;
    }

    writeln!(out, "Already in place: {}", summary.already)?;
    writeln!(
        out,
        "Skipped: {} no origin, {} unsupported remote, {} excluded, {} conflicts, {} duplicates, {} linked worktrees, {} unparsable, {} root repo",
        summary.no_origin,
        summary.unsupported,
        summary.excluded,
        summary.conflicts,
        summary.duplicates,
        summary.worktrees,
        summary.unparsable,
        summary.root
    )?;

    if summary.failed > 0 {
        writeln!(out, "Failed: {}", summary.failed)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{find_repos, is_excluded, run, Format, Options, Status};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn unique_root(name: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        root.push(format!("git_mirror_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create temp root");
        root
    }

    fn init_repo(path: &Path, origin: Option<&str>) {
        fs::create_dir_all(path).expect("create repo dir");
        let status = Command::new("git")
            .args(["init", "-q"])
            .arg(path)
            .status()
            .expect("run git init");
        assert!(status.success(), "git init failed for {}", path.display());

        if let Some(origin) = origin {
            let status = Command::new("git")
                .arg("-C")
                .arg(path)
                .args(["remote", "add", "origin", origin])
                .status()
                .expect("run git remote add");
            assert!(
                status.success(),
                "git remote add failed for {}",
                path.display()
            );
        }
    }

    fn has_git() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn options(root: &Path) -> Options {
        Options {
            root: root.to_path_buf(),
            apply: false,
            include_nested: false,
            force: false,
            full_host: false,
            excludes: Vec::new(),
            format: Format::Tsv,
        }
    }

    fn tsv_rows(output: &[u8]) -> Vec<Vec<String>> {
        String::from_utf8_lossy(output)
            .lines()
            .skip(1)
            .filter(|line| !line.is_empty() && !line.starts_with("Dry run"))
            .map(|line| line.split('\t').map(str::to_string).collect())
            .collect()
    }

    #[test]
    fn test_is_excluded() {
        let root = PathBuf::from("/projects");
        let excludes = vec![PathBuf::from("/projects/go")];
        assert!(is_excluded(Path::new("/projects/go/src"), &excludes));
        assert!(is_excluded(Path::new("/projects/go"), &excludes));
        assert!(!is_excluded(Path::new("/projects/github"), &excludes));
        assert!(!is_excluded(Path::new("/projects/gopher"), &excludes));
    }

    #[test]
    fn test_find_repos_deepest_first() {
        let root = unique_root("find");
        let parent = root.join("parent");
        let child = parent.join("child");
        init_repo(&parent, None);
        init_repo(&child, None);

        let repos = find_repos(&root, true).expect("find");
        assert_eq!(repos, vec![child.clone(), parent.clone()]);

        // By default nested repos are not scanned at all.
        let repos = find_repos(&root, false).expect("find");
        assert_eq!(repos, vec![parent.clone()]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_migrate_dry_run_and_apply() {
        if !has_git() {
            return;
        }

        let root = unique_root("migrate");

        init_repo(
            &root.join("old/a/repo1"),
            Some("git@bitbucket.org:pplcs/repo1.git"),
        );
        init_repo(
            &root.join("old/b/repo2"),
            Some("https://gitlab.com/group/sub/project.git"),
        );
        init_repo(
            &root.join("old/c/duplicate"),
            Some("git@bitbucket.org:pplcs/repo1.git"),
        );
        init_repo(
            &root.join("old/d/googlesource"),
            Some("https://go.googlesource.com/crypto"),
        );
        init_repo(&root.join("old/e/no-origin"), None);
        init_repo(
            &root.join("old/f/parent"),
            Some("git@github.com:org/parent.git"),
        );
        init_repo(
            &root.join("old/f/parent/kid"),
            Some("git@github.com:org/kid.git"),
        );
        init_repo(
            &root.join("old/g/conflict"),
            Some("git@github.com:org/conflict.git"),
        );
        init_repo(
            &root.join("github/org/conflict"),
            Some("git@github.com:other/conflict.git"),
        );
        init_repo(
            &root.join("github/org/ok"),
            Some("git@github.com:org/ok.git"),
        );

        // Dry run: nothing moves.
        let mut out = Vec::new();
        let mut err = Vec::new();
        let summary = run(&options(&root), &mut out, &mut err).expect("dry run");
        assert_eq!(summary.planned, 5);
        assert_eq!(summary.duplicates, 1);
        assert_eq!(summary.no_origin, 1);
        assert_eq!(summary.conflicts, 1);
        assert_eq!(summary.already, 1);
        assert!(root.join("old/a/repo1").is_dir());

        let rows = tsv_rows(&out);
        let row = |path: &str| {
            rows.iter()
                .find(|row| row[1].ends_with(path))
                .unwrap_or_else(|| panic!("missing row for {path}: {rows:?}"))
        };

        assert_eq!(row("old/a/repo1")[0], "move");
        assert_eq!(
            row("old/a/repo1")[3],
            root.join("bitbucket/pplcs/repo1").display().to_string()
        );
        assert_eq!(
            row("old/d/googlesource")[3],
            root.join("googlesource/crypto").display().to_string()
        );
        assert_eq!(row("old/c/duplicate")[0], "duplicate");
        assert_eq!(row("old/e/no-origin")[0], "no-origin");
        assert_eq!(row("old/g/conflict")[0], "conflict");
        assert_eq!(row("github/org/ok")[0], "ok");

        // Apply.
        let mut opts = options(&root);
        opts.apply = true;
        let mut out = Vec::new();
        let mut err = Vec::new();
        let summary = run(&opts, &mut out, &mut err).expect("apply");
        // The misplaced `github/org/conflict` moves out of the way first, so
        // `old/g/conflict` can then take its place in the same run.
        assert_eq!(summary.moved, 6);
        assert!(root.join("bitbucket/pplcs/repo1").is_dir());
        assert!(root.join("gitlab/group/sub/project").is_dir());
        assert!(root.join("googlesource/crypto").is_dir());
        assert!(root.join("github/org/parent").is_dir());
        assert!(root.join("github/other/conflict").is_dir());
        assert!(root.join("github/org/conflict").is_dir());
        assert!(!root.join("old/a").exists());
        assert!(!root.join("old/b").exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_status_names() {
        assert_eq!(Status::Move.as_str(), "move");
        assert_eq!(Status::Moved.as_str(), "moved");
        assert_eq!(Status::NoOrigin.as_str(), "no-origin");
        assert_eq!(Status::Unsupported.as_str(), "unsupported-remote");
        assert_eq!(Status::Root.as_str(), "root");
    }
}
