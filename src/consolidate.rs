use std::fs as stdfs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

use crate::git;

/// One ref (or stash entry) to import into the keeper.
#[derive(Clone, Debug)]
pub struct RefImport {
    /// Source ref: `refs/heads/<name>` for branches, or a stash label such as
    /// `stash@{0}`.
    pub source: String,
    /// Path component below `migration/<date>/<slug>/`.
    pub name: String,
    pub oid: String,
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub source: PathBuf,
    pub date: String,
    pub slug: String,
    pub archive_dir: PathBuf,
    pub branches: Vec<RefImport>,
    pub stashes: Vec<RefImport>,
    pub detached: Option<RefImport>,
    pub dirty: usize,
    pub untracked: usize,
}

impl Plan {
    pub fn planned_detail(&self, keeper: &Path) -> String {
        let mut parts = vec![
            format!("into {}", keeper.display()),
            format!("{} branch(es)", self.branches.len()),
        ];

        if !self.stashes.is_empty() {
            parts.push(format!("{} stash(es)", self.stashes.len()));
        }
        if self.detached.is_some() {
            parts.push("detached HEAD".to_string());
        }
        if self.dirty > 0 {
            parts.push(format!("{} dirty", self.dirty));
        }
        if self.untracked > 0 {
            parts.push(format!("{} untracked", self.untracked));
        }

        parts.push(format!("archive {}", self.archive_dir.display()));
        parts.join(", ")
    }
}

#[derive(Clone, Debug)]
pub struct Execution {
    pub imported: usize,
    pub note: Option<String>,
}

/// UTC date used in migration refs and archive paths (`YYYY-MM-DD`).
pub fn today_utc() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    civil_from_days((seconds / 86_400) as i64)
}

fn civil_from_days(days: i64) -> String {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };

    format!("{year:04}-{month:02}-{day:02}")
}

/// Short, ref-safe slug for a checkout path.
pub fn slug_for(path: &Path, root: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut slug = String::new();

    for character in relative.to_string_lossy().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if matches!(character, '/' | '-' | '_') {
            slug.push('-');
        }
    }

    let slug = slug.trim_matches('-');
    let mut slug = if slug.is_empty() {
        "source".to_string()
    } else {
        slug.to_string()
    };

    if slug.len() > 80 {
        slug.truncate(80);
        slug = slug.trim_end_matches('-').to_string();
    }

    slug
}

/// Avoid clobbering an archive from an earlier run.
pub fn unique_archive_dir(base: &Path) -> PathBuf {
    if !base.exists() {
        return base.to_path_buf();
    }

    let Some(name) = base.file_name() else {
        return base.to_path_buf();
    };

    for suffix in 2..1_000 {
        let candidate = base.with_file_name(format!("{}-{suffix}", name.to_string_lossy()));
        if !candidate.exists() {
            return candidate;
        }
    }

    base.to_path_buf()
}

/// Work out what would need to move from `source` into the keeper.
pub fn plan(
    source: &Path,
    date: &str,
    slug: &str,
    archive_dir: &Path,
    dirty: usize,
    untracked: usize,
) -> Result<Plan> {
    Ok(Plan {
        source: source.to_path_buf(),
        date: date.to_string(),
        slug: slug.to_string(),
        archive_dir: archive_dir.to_path_buf(),
        branches: unique_branches(source)?,
        stashes: stashes(source)?,
        detached: detached_head(source)?,
        dirty,
        untracked,
    })
}

fn unique_branches(source: &Path) -> Result<Vec<RefImport>> {
    let mut imports = Vec::new();

    let Some(output) = git::git_stdout(
        source,
        &[
            "for-each-ref",
            "--format=%(refname:short)%00%(objectname)",
            "refs/heads",
        ],
    )?
    else {
        return Ok(imports);
    };

    for line in output.lines() {
        let mut fields = line.split('\0');
        let Some(name) = fields.next() else {
            continue;
        };
        let Some(oid) = fields.next() else {
            continue;
        };

        if name.is_empty() || oid.is_empty() {
            continue;
        }

        if has_unique_commits(source, name) {
            imports.push(RefImport {
                source: format!("refs/heads/{name}"),
                name: name.to_string(),
                oid: oid.to_string(),
            });
        }
    }

    Ok(imports)
}

fn has_unique_commits(source: &Path, branch: &str) -> bool {
    let count = |args: &[&str]| {
        git::git_stdout(source, args)
            .ok()
            .flatten()
            .and_then(|output| output.trim().parse::<usize>().ok())
    };

    count(&["rev-list", "--count", branch, "--not", "--remotes=origin"])
        .or_else(|| count(&["rev-list", "--count", branch]))
        .unwrap_or(0)
        > 0
}

fn stashes(source: &Path) -> Result<Vec<RefImport>> {
    let mut imports = Vec::new();

    let Some(list) = git::git_stdout(source, &["stash", "list", "--format=%gd"])? else {
        return Ok(imports);
    };

    for (index, entry) in list.lines().enumerate() {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        if let Some(oid) = git::git_stdout(source, &["rev-parse", entry])? {
            if !oid.is_empty() {
                imports.push(RefImport {
                    source: entry.to_string(),
                    name: format!("stash-{index}"),
                    oid,
                });
            }
        }
    }

    Ok(imports)
}

fn detached_head(source: &Path) -> Result<Option<RefImport>> {
    if git::git_stdout(source, &["symbolic-ref", "-q", "HEAD"])?.is_some() {
        return Ok(None);
    }

    let Some(oid) = git::git_stdout(source, &["rev-parse", "--verify", "HEAD"])? else {
        return Ok(None);
    };
    if oid.is_empty() {
        return Ok(None);
    }

    let covered = git::git_stdout(
        source,
        &["branch", "--contains", &oid, "--format=%(refname:short)"],
    )?
    .map(|output| !output.trim().is_empty())
    .unwrap_or(false);

    if covered {
        return Ok(None);
    }

    Ok(Some(RefImport {
        source: "HEAD".to_string(),
        name: format!("detached-{}", short(&oid)),
        oid,
    }))
}

/// Import the plan's refs/state into `keeper`, archive the source checkout, and
/// verify every imported ref.
pub fn execute(plan: &Plan, keeper: &Path, root: &Path, has_nested: bool) -> Result<Execution> {
    stdfs::create_dir_all(&plan.archive_dir)
        .with_context(|| format!("cannot create {}", plan.archive_dir.display()))?;

    let mut manifest = String::new();
    manifest.push_str(&format!("date: {}\n", plan.date));
    manifest.push_str(&format!("slug: {}\n", plan.slug));
    manifest.push_str(&format!("source: {}\n", plan.source.display()));
    manifest.push_str(&format!("keeper: {}\n", keeper.display()));
    manifest.push_str("refs:\n");

    let mut imported = 0;
    for import in plan
        .branches
        .iter()
        .chain(plan.stashes.iter())
        .chain(plan.detached.iter())
    {
        let destination = import_ref(plan, keeper, import)?;
        imported += 1;
        manifest.push_str(&format!(
            "  {} -> {} ({})\n",
            import.source,
            destination,
            short(&import.oid)
        ));
    }

    if plan.dirty > 0 {
        write_worktree_patch(&plan.source, &plan.archive_dir)?;
        manifest.push_str(&format!("dirty: {} (worktree.patch)\n", plan.dirty));
    } else {
        manifest.push_str("dirty: 0\n");
    }

    if plan.untracked > 0 {
        let copied = export_untracked(&plan.source, &plan.archive_dir.join("untracked"))?;
        manifest.push_str(&format!("untracked: {} files\n", copied));
    } else {
        manifest.push_str("untracked: 0\n");
    }

    let archived = if has_nested {
        manifest.push_str("archived: false (contains nested repositories)\n");
        false
    } else {
        let destination = plan.archive_dir.join("source");
        stdfs::rename(&plan.source, &destination).with_context(|| {
            format!(
                "cannot archive {} to {}",
                plan.source.display(),
                destination.display()
            )
        })?;
        crate::fs::prune_empty_parents(&plan.source, root);
        manifest.push_str("archived: true (source/)\n");
        true
    };

    let manifest_path = plan.archive_dir.join("manifest.txt");
    stdfs::write(&manifest_path, manifest)
        .with_context(|| format!("cannot write {}", manifest_path.display()))?;

    Ok(Execution {
        imported,
        note: if archived {
            None
        } else {
            Some("source left in place (contains nested repositories)".to_string())
        },
    })
}

fn import_ref(plan: &Plan, keeper: &Path, import: &RefImport) -> Result<String> {
    let base = format!(
        "refs/heads/migration/{}/{}/{}",
        plan.date, plan.slug, import.name
    );
    let destination = free_ref(keeper, &base, &import.oid)?;

    if let Some(existing) = ref_oid(keeper, &destination)? {
        if existing == import.oid {
            return Ok(destination);
        }
    }

    if import.source == "HEAD" || import.source.starts_with("stash@{") {
        let temporary = format!("refs/migration-export/{}-{}", plan.slug, import.name);
        update_ref(&plan.source, &temporary, &import.oid)?;
        let result = fetch_ref(keeper, &plan.source, &temporary, &destination);
        let _ = delete_ref(&plan.source, &temporary);
        result?;
    } else {
        fetch_ref(keeper, &plan.source, &import.source, &destination)?;
    }

    match ref_oid(keeper, &destination)? {
        Some(oid) if oid == import.oid => Ok(destination),
        Some(oid) => bail!(
            "imported ref {} points to {}, expected {}",
            destination,
            short(&oid),
            short(&import.oid)
        ),
        None => bail!("imported ref {} is missing", destination),
    }
}

fn free_ref(keeper: &Path, base: &str, oid: &str) -> Result<String> {
    match ref_oid(keeper, base)? {
        None => Ok(base.to_string()),
        Some(existing) if existing == oid => Ok(base.to_string()),
        Some(_) => {
            for suffix in 2..1_000 {
                let candidate = format!("{base}-{suffix}");
                match ref_oid(keeper, &candidate)? {
                    None => return Ok(candidate),
                    Some(existing) if existing == oid => return Ok(candidate),
                    Some(_) => {}
                }
            }

            bail!("cannot find a free migration ref for {base}")
        }
    }
}

fn ref_oid(repo: &Path, name: &str) -> Result<Option<String>> {
    Ok(
        git::git_stdout(repo, &["rev-parse", "--verify", "--quiet", name])?
            .filter(|oid| !oid.is_empty()),
    )
}

fn update_ref(repo: &Path, name: &str, oid: &str) -> Result<()> {
    run_status(repo, &["update-ref", name, oid])
}

fn delete_ref(repo: &Path, name: &str) -> Result<()> {
    run_status(repo, &["update-ref", "-d", name])
}

fn run_status(repo: &Path, args: &[&str]) -> Result<()> {
    let output = git::run(repo, args)?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn fetch_ref(keeper: &Path, source: &Path, source_ref: &str, destination: &str) -> Result<()> {
    let source = source.to_string_lossy();
    let refspec = format!("+{source_ref}:{destination}");
    let output = git::run(
        keeper,
        &["fetch", "--no-tags", "--quiet", &source, &refspec],
    )?;

    if !output.status.success() {
        bail!(
            "git fetch {source_ref} -> {destination} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

fn write_worktree_patch(source: &Path, archive_dir: &Path) -> Result<()> {
    let head = git::git_stdout(source, &["rev-parse", "--verify", "HEAD"])?;

    if head.is_some() {
        let output = git::run(source, &["diff", "--binary", "HEAD"])?;
        if !output.status.success() {
            bail!(
                "git diff failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        stdfs::write(archive_dir.join("worktree.patch"), &output.stdout)?;
    } else {
        let worktree = git::run(source, &["diff", "--binary"])?;
        stdfs::write(archive_dir.join("worktree.patch"), &worktree.stdout)?;

        let index = git::run(source, &["diff", "--binary", "--cached"])?;
        stdfs::write(archive_dir.join("index.patch"), &index.stdout)?;
    }

    Ok(())
}

fn export_untracked(source: &Path, destination: &Path) -> Result<usize> {
    let Some(list) = git::git_stdout(
        source,
        &["ls-files", "-z", "--others", "--exclude-standard"],
    )?
    else {
        return Ok(0);
    };

    let mut copied = 0;
    for relative in list.split('\0').filter(|entry| !entry.is_empty()) {
        let from = source.join(relative);
        let to = destination.join(relative);

        if let Some(parent) = to.parent() {
            stdfs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }

        if from.is_file() {
            stdfs::copy(&from, &to)
                .with_context(|| format!("cannot copy {} to {}", from.display(), to.display()))?;
            copied += 1;
        }
    }

    Ok(copied)
}

fn short(oid: &str) -> &str {
    oid.get(..8).unwrap_or(oid)
}

#[cfg(test)]
mod tests {
    use super::{civil_from_days, slug_for};
    use std::path::Path;

    #[test]
    fn test_civil_from_days_epoch() {
        assert_eq!(civil_from_days(0), "1970-01-01");
    }

    #[test]
    fn test_civil_from_days_known_date() {
        // 2026-11-28 is 20,785 days after the Unix epoch.
        assert_eq!(civil_from_days(20_785), "2026-11-28");
    }

    #[test]
    fn test_slug_for() {
        assert_eq!(
            slug_for(
                Path::new("/projects/novamedia/Atlas/aws_authenticator"),
                Path::new("/projects")
            ),
            "novamedia-atlas-aws-authenticator"
        );
    }
}
