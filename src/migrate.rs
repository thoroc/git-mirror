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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ReportFormat {
    #[default]
    Csv,
    Tsv,
    Json,
    Markdown,
    Text,
}

impl ReportFormat {
    /// Infer the report format from a file extension. Unknown or missing
    /// extensions fall back to CSV.
    pub fn from_path(path: &Path) -> Self {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        match extension.as_deref() {
            Some("csv") => ReportFormat::Csv,
            Some("tsv") | Some("tab") => ReportFormat::Tsv,
            Some("json") => ReportFormat::Json,
            Some("md") | Some("markdown") => ReportFormat::Markdown,
            Some("txt") | Some("text") => ReportFormat::Text,
            _ => ReportFormat::Csv,
        }
    }
}

impl std::fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ReportFormat::Csv => "csv",
            ReportFormat::Tsv => "tsv",
            ReportFormat::Json => "json",
            ReportFormat::Markdown => "markdown",
            ReportFormat::Text => "text",
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
    Failed,
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
            Status::Failed => "failed",
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
            Status::Failed => self.failed += 1,
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
    pub report: Option<PathBuf>,
    pub report_format: ReportFormat,
}

/// Classify every repository under the root, optionally moving the ones that
/// need it. Returns the summary; individual move failures are reported but do
/// not abort the run.
pub fn run(opts: &Options, out: &mut dyn Write, err: &mut dyn Write) -> Result<Summary> {
    let color = opts.format == Format::Text && std::io::stdout().is_terminal();
    let repos = find_repos(&opts.root, opts.include_nested)?;
    let mut summary = Summary::default();
    let mut claimed: HashMap<PathBuf, (String, PathBuf)> = HashMap::new();
    let mut entries = Vec::new();

    if opts.format == Format::Tsv {
        writeln!(out, "status\tcurrent\torigin\ttarget\tdetail")?;
    }

    for current in repos {
        let mut entry = classify(&current, opts, &mut claimed)?;

        if entry.status == Status::Move && opts.apply {
            match apply_move(&entry, &opts.root) {
                Ok(()) => entry.status = Status::Moved,
                Err(error) => {
                    entry.status = Status::Failed;
                    entry.detail = Some(format!("{error:#}"));
                    summary.record(entry.status);
                    if opts.format == Format::Tsv {
                        emit(&entry, opts.format, color, out)?;
                    }
                    writeln!(err, "{}", text_line(&entry, color))?;
                    entries.push(entry);
                    continue;
                }
            }
        }

        summary.record(entry.status);
        emit(&entry, opts.format, color, out)?;
        entries.push(entry);
    }

    if opts.format == Format::Tsv {
        print_summary(&summary, opts.apply, err)?;
    } else {
        print_summary(&summary, opts.apply, out)?;
    }

    if let Some(report) = &opts.report {
        write_report(report, opts.report_format, &entries, &summary, opts)?;
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

fn tsv_field(value: &str) -> String {
    value
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn tsv_row(entry: &Entry) -> String {
    let target = entry
        .target
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();

    format!(
        "{}\t{}\t{}\t{}\t{}",
        entry.status.as_str(),
        tsv_field(&entry.current.display().to_string()),
        tsv_field(entry.origin.as_deref().unwrap_or("")),
        tsv_field(&target),
        tsv_field(entry.detail.as_deref().unwrap_or(""))
    )
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn csv_row(entry: &Entry) -> String {
    let current = entry.current.display().to_string();
    let target = entry
        .target
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();

    [
        entry.status.as_str(),
        current.as_str(),
        entry.origin.as_deref().unwrap_or(""),
        target.as_str(),
        entry.detail.as_deref().unwrap_or(""),
    ]
    .iter()
    .copied()
    .map(csv_field)
    .collect::<Vec<_>>()
    .join(",")
}

fn markdown_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "<br>")
        .replace('\r', "")
}

fn markdown_row(entry: &Entry) -> String {
    let current = entry.current.display().to_string();
    let target = entry
        .target
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();

    format!(
        "| {} | {} | {} | {} | {} |",
        markdown_field(entry.status.as_str()),
        markdown_field(&current),
        markdown_field(entry.origin.as_deref().unwrap_or("")),
        markdown_field(&target),
        markdown_field(entry.detail.as_deref().unwrap_or(""))
    )
}

fn emit(entry: &Entry, format: Format, color: bool, out: &mut dyn Write) -> Result<()> {
    if format == Format::Tsv {
        writeln!(out, "{}", tsv_row(entry))?;
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
        Status::Failed => format!(
            "{} {current} -> {target}: {detail}",
            paint("FAIL", RED, color)
        ),
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

fn write_report(
    path: &Path,
    format: ReportFormat,
    entries: &[Entry],
    summary: &Summary,
    opts: &Options,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            stdfs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
    }

    let mut file = stdfs::File::create(path)
        .with_context(|| format!("cannot create report {}", path.display()))?;

    match format {
        ReportFormat::Text => {
            for entry in entries {
                writeln!(file, "{}", text_line(entry, false))?;
            }
            print_summary(summary, opts.apply, &mut file)?;
        }
        ReportFormat::Csv => {
            writeln!(file, "status,current,origin,target,detail")?;
            for entry in entries {
                writeln!(file, "{}", csv_row(entry))?;
            }
        }
        ReportFormat::Tsv => {
            writeln!(file, "status\tcurrent\torigin\ttarget\tdetail")?;
            for entry in entries {
                writeln!(file, "{}", tsv_row(entry))?;
            }
        }
        ReportFormat::Json => write_json_report(&mut file, entries, summary, opts)?,
        ReportFormat::Markdown => {
            writeln!(file, "| Status | Current | Origin | Target | Detail |")?;
            writeln!(file, "| --- | --- | --- | --- | --- |")?;
            for entry in entries {
                writeln!(file, "{}", markdown_row(entry))?;
            }
            write_markdown_summary(&mut file, summary, opts.apply)?;
        }
    }

    Ok(())
}

fn write_markdown_summary<W: Write>(out: &mut W, summary: &Summary, apply: bool) -> Result<()> {
    writeln!(out, "\n## Summary\n")?;

    if apply {
        writeln!(out, "- **Moved:** {}", summary.moved)?;
    } else {
        writeln!(
            out,
            "- **Dry run:** {} repo(s) would move. Re-run with `--apply` to perform the moves.",
            summary.planned
        )?;
    }

    writeln!(out, "- **Already in place:** {}", summary.already)?;
    writeln!(
        out,
        "- **Skipped:** {} no origin, {} unsupported remote, {} excluded, {} conflicts, {} duplicates, {} linked worktrees, {} unparsable, {} root repo",
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
        writeln!(out, "- **Failed:** {}", summary.failed)?;
    }

    Ok(())
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if (c as u32) < 0x20 => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }

    escaped.push('"');
    escaped
}

fn json_optional(value: Option<&str>) -> String {
    value.map(json_string).unwrap_or_else(|| "null".to_string())
}

fn write_json_report<W: Write>(
    out: &mut W,
    entries: &[Entry],
    summary: &Summary,
    opts: &Options,
) -> Result<()> {
    writeln!(out, "{{")?;
    writeln!(out, "  \"schema_version\": 1,")?;
    writeln!(
        out,
        "  \"root\": {},",
        json_string(&opts.root.display().to_string())
    )?;
    writeln!(out, "  \"apply\": {},", opts.apply)?;
    writeln!(out, "  \"summary\": {{")?;
    writeln!(out, "    \"moved\": {},", summary.moved)?;
    writeln!(out, "    \"planned\": {},", summary.planned)?;
    writeln!(out, "    \"already\": {},", summary.already)?;
    writeln!(out, "    \"no_origin\": {},", summary.no_origin)?;
    writeln!(out, "    \"unsupported\": {},", summary.unsupported)?;
    writeln!(out, "    \"excluded\": {},", summary.excluded)?;
    writeln!(out, "    \"conflicts\": {},", summary.conflicts)?;
    writeln!(out, "    \"duplicates\": {},", summary.duplicates)?;
    writeln!(out, "    \"worktrees\": {},", summary.worktrees)?;
    writeln!(out, "    \"unparsable\": {},", summary.unparsable)?;
    writeln!(out, "    \"root_repos\": {},", summary.root)?;
    writeln!(out, "    \"failed\": {}", summary.failed)?;
    writeln!(out, "  }},")?;
    writeln!(out, "  \"repositories\": [")?;

    for (index, entry) in entries.iter().enumerate() {
        let target = entry.target.as_ref().map(|path| path.display().to_string());

        writeln!(out, "    {{")?;
        writeln!(
            out,
            "      \"status\": {},",
            json_string(entry.status.as_str())
        )?;
        writeln!(
            out,
            "      \"current\": {},",
            json_string(&entry.current.display().to_string())
        )?;
        writeln!(
            out,
            "      \"origin\": {},",
            json_optional(entry.origin.as_deref())
        )?;
        writeln!(
            out,
            "      \"target\": {},",
            json_optional(target.as_deref())
        )?;
        writeln!(
            out,
            "      \"detail\": {}",
            json_optional(entry.detail.as_deref())
        )?;
        writeln!(
            out,
            "    }}{}",
            if index + 1 == entries.len() { "" } else { "," }
        )?;
    }

    writeln!(out, "  ]")?;
    writeln!(out, "}}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        find_repos, is_excluded, run, write_report, Entry, Format, Options, ReportFormat, Status,
        Summary,
    };
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
            report: None,
            report_format: ReportFormat::Tsv,
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
    fn test_write_report_formats() {
        let root = unique_root("report");
        let entries = vec![
            Entry {
                status: Status::Move,
                current: root.join("old/repo"),
                origin: Some("git@github.com:org/repo.git".to_string()),
                target: Some(root.join("github/org/repo")),
                detail: None,
            },
            Entry {
                status: Status::Duplicate,
                current: root.join("old/dup"),
                origin: Some("git@github.com:org/repo.git".to_string()),
                target: Some(root.join("github/org/repo")),
                detail: Some(root.join("old/repo").display().to_string()),
            },
        ];
        let summary = Summary {
            planned: 1,
            duplicates: 1,
            ..Summary::default()
        };
        let opts = options(&root);

        let tsv = root.join("report.tsv");
        write_report(&tsv, ReportFormat::Tsv, &entries, &summary, &opts).expect("tsv report");
        let tsv_text = fs::read_to_string(&tsv).expect("read tsv");
        assert!(tsv_text.starts_with("status\tcurrent\torigin\ttarget\tdetail\n"));
        assert!(tsv_text.contains("move\t"));
        assert!(tsv_text.contains("duplicate\t"));

        let csv = root.join("report.csv");
        write_report(&csv, ReportFormat::Csv, &entries, &summary, &opts).expect("csv report");
        let csv_text = fs::read_to_string(&csv).expect("read csv");
        assert!(csv_text.starts_with("status,current,origin,target,detail\n"));
        assert!(csv_text.contains("move,"));
        assert!(csv_text.contains("duplicate,"));

        let markdown = root.join("report.md");
        write_report(&markdown, ReportFormat::Markdown, &entries, &summary, &opts)
            .expect("markdown report");
        let markdown_text = fs::read_to_string(&markdown).expect("read markdown");
        assert!(markdown_text.starts_with("| Status | Current | Origin | Target | Detail |\n"));
        assert!(markdown_text.contains("## Summary"));
        assert!(markdown_text.contains("**Dry run:** 1 repo(s) would move."));

        let json = root.join("report.json");
        write_report(&json, ReportFormat::Json, &entries, &summary, &opts).expect("json report");
        let json_text = fs::read_to_string(&json).expect("read json");
        assert!(json_text.contains("\"schema_version\": 1"));
        assert!(json_text.contains("\"planned\": 1"));
        assert!(json_text.contains("\"status\": \"move\""));
        assert!(json_text.contains("\"detail\": null"));

        let text = root.join("report.txt");
        write_report(&text, ReportFormat::Text, &entries, &summary, &opts).expect("text report");
        let text_text = fs::read_to_string(&text).expect("read text");
        assert!(text_text.contains("PLAN"));
        assert!(text_text.contains("Dry run: 1 repo(s) would move."));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_json_string_escapes() {
        assert_eq!(
            super::json_string("a\"b\\c\n"),
            "\"a\\\"b\\\\c\\n\"".to_string()
        );
    }

    #[test]
    fn test_report_format_from_path() {
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.csv")),
            ReportFormat::Csv
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.CSV")),
            ReportFormat::Csv
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.tsv")),
            ReportFormat::Tsv
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.tab")),
            ReportFormat::Tsv
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.json")),
            ReportFormat::Json
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.md")),
            ReportFormat::Markdown
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.markdown")),
            ReportFormat::Markdown
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.txt")),
            ReportFormat::Text
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan")),
            ReportFormat::Csv
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("plan.unknown")),
            ReportFormat::Csv
        );
    }

    #[test]
    fn test_csv_field_quotes() {
        assert_eq!(super::csv_field("plain"), "plain");
        assert_eq!(super::csv_field("a,b"), "\"a,b\"");
        assert_eq!(super::csv_field("a\"b"), "\"a\"\"b\"");
        assert_eq!(super::csv_field("a\nb"), "\"a\nb\"");
    }

    #[test]
    fn test_markdown_field_escapes() {
        assert_eq!(super::markdown_field("a|b"), "a\\|b");
        assert_eq!(super::markdown_field("a\nb"), "a<br>b");
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
        let report = root.join("plan.tsv");
        let mut dry = options(&root);
        dry.report = Some(report.clone());
        let mut out = Vec::new();
        let mut err = Vec::new();
        let summary = run(&dry, &mut out, &mut err).expect("dry run");
        assert_eq!(summary.planned, 5);
        assert_eq!(summary.duplicates, 1);
        assert_eq!(summary.no_origin, 1);
        assert_eq!(summary.conflicts, 1);
        assert_eq!(summary.already, 1);
        assert!(root.join("old/a/repo1").is_dir());

        let plan = fs::read_to_string(&report).expect("read report");
        assert!(plan.starts_with("status\tcurrent\torigin\ttarget\tdetail\n"));
        assert!(plan.lines().any(|line| line.starts_with("move\t")));

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
