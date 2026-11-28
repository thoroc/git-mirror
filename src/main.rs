mod fs;
mod git;
mod migrate;
mod util;

use clap::{Args, Parser, Subcommand};
use dialoguer::Confirm;
use owo_colors::OwoColorize;
use std::env;
use std::ffi::OsString;
use std::process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Clone (or update) a repository into the projects tree
    Clone(CloneArgs),
    /// Move existing repositories into the projects tree based on their origin remote
    Migrate(MigrateArgs),
}

#[derive(Args)]
struct CloneArgs {
    /// Git repository URL
    repo: String,

    /// Root directory where projects are stored
    #[arg(short, long, default_value_t = String::from("~/Projects"))]
    root: String,

    /// Print a shell-friendly cd command instead of executing
    #[arg(long)]
    print_cd: bool,

    /// Dry run: show commands without executing
    #[arg(long)]
    dry_run: bool,

    /// Open the repo in VS Code after cloning or when it exists
    #[arg(long)]
    open_vs_code: bool,

    /// Do not open the repo in VS Code
    #[arg(long)]
    no_open_vs_code: bool,

    /// Disable interactive prompts (useful in CI)
    #[arg(long)]
    no_prompt: bool,

    /// Use full host domain in local path (e.g. `github.com` instead of `github`)
    #[arg(long)]
    full_host: bool,
}

#[derive(Args)]
struct MigrateArgs {
    /// Root directory where projects are stored
    #[arg(short, long, default_value_t = String::from("~/Projects"))]
    root: String,

    /// Perform the moves (default is a dry run)
    #[arg(long)]
    apply: bool,

    /// Output format: text or tsv
    #[arg(long, value_enum, default_value_t = migrate::Format::Text)]
    format: migrate::Format,

    /// Also scan inside repositories for nested repositories (including vendored clones)
    #[arg(long)]
    include_nested: bool,

    /// Move repos that have linked worktrees (those worktrees will break)
    #[arg(long)]
    force: bool,

    /// Skip repositories at or under this path prefix (repeatable)
    #[arg(long = "exclude", value_name = "PREFIX")]
    exclude: Vec<String>,

    /// Use full host domain in the target path (e.g. `github.com` instead of `github`)
    #[arg(long)]
    full_host: bool,
}

/// Keep the historical `git-mirror <repo>` invocation working by treating it
/// as `git-mirror clone <repo>`. Explicit subcommands and help/version flags
/// are left untouched.
fn normalize_args<I, T>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut args: Vec<OsString> = args.into_iter().map(Into::into).collect();

    if args.len() >= 2 {
        let first = args[1].to_string_lossy();
        let known = matches!(
            first.as_ref(),
            "clone" | "migrate" | "-h" | "--help" | "-V" | "--version"
        );
        if !known {
            args.insert(1, OsString::from("clone"));
        }
    }

    args
}

fn main() {
    let cli = Cli::parse_from(normalize_args(env::args_os()));

    match cli.command {
        Command::Clone(args) => run_clone(args),
        Command::Migrate(args) => run_migrate(args),
    }
}

fn run_migrate(args: MigrateArgs) {
    let root = match fs::expand_root(&args.root) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("Error resolving projects root: {error}");
            process::exit(2);
        }
    };

    if !root.is_dir() {
        eprintln!("error: projects root not found: {}", root.display());
        process::exit(2);
    }

    let mut excludes = Vec::with_capacity(args.exclude.len());
    for prefix in &args.exclude {
        match fs::expand_root(prefix) {
            Ok(path) => excludes.push(path),
            Err(error) => {
                eprintln!("Error resolving exclude prefix {prefix}: {error}");
                process::exit(2);
            }
        }
    }

    let options = migrate::Options {
        root,
        apply: args.apply,
        include_nested: args.include_nested,
        force: args.force,
        full_host: args.full_host,
        excludes,
        format: args.format,
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let stderr = std::io::stderr();
    let mut err = stderr.lock();

    match migrate::run(&options, &mut out, &mut err) {
        Ok(summary) => process::exit(if summary.failed > 0 { 1 } else { 0 }),
        Err(error) => {
            eprintln!("Error: {error:#}");
            process::exit(1);
        }
    }
}

fn run_clone(args: CloneArgs) {
    let local = match fs::build_local_repo_path(&args.root, &args.repo, args.full_host) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", format!("Error building local path: {}", e).red());
            process::exit(1);
        }
    };

    // If the user only wants the cd command, print it and exit early
    if args.print_cd {
        println!("{}", format!("cd \"{}\"", local.display()).green());
        return;
    }

    // Decide whether to open VS Code: explicit flags take precedence,
    // otherwise prompt the user. If running in CI or `--no-prompt` is set, do not prompt.
    let is_ci = env::var("CI").is_ok();
    let decide_open = |default: bool| -> bool {
        // Always open if explicitly requested
        if args.open_vs_code {
            return true;
        }

        // Never open if explicitly disabled
        if args.no_open_vs_code {
            return false;
        }

        // Never open in CI environments
        if is_ci {
            return false;
        }

        // Use default without prompting if --no-prompt is set
        if args.no_prompt {
            return default;
        }

        // Otherwise, prompt the user
        Confirm::new()
            .with_prompt("Open the repository in VS Code?")
            .default(default)
            .interact()
            .unwrap_or(default)
    };

    // If the repo already exists locally, fetch updates
    if git::repo_exists(&local) {
        if args.dry_run {
            println!(
                "{}",
                format!("> Dry run: Fetching repository: {}", local.display()).yellow()
            );
        } else {
            if let Err(e) = git::fetch_repo(&local, args.dry_run) {
                eprintln!("{}", format!("Error fetching repo: {}", e).red());
                process::exit(1);
            }
            println!(
                "{}",
                format!("Fetched repository at {}", local.display()).green()
            );
        }

        let open = decide_open(true);
        if open {
            if let Err(e) = git::open_in_vscode(&local, args.dry_run) {
                eprintln!(
                    "{}",
                    format!("Warning: failed to open VS Code: {}", e).yellow()
                );
            }
        }

        println!(
            "{}",
            format!(
                "To move to the project's directory, please run: \"cd {}\"",
                local.display()
            )
            .cyan()
        );
        return;
    }

    // Repo doesn't exist: clone it
    if let Err(e) = git::clone_repo(&args.repo, &local, args.dry_run) {
        eprintln!("{}", format!("Error cloning repo: {}", e).red());
        process::exit(1);
    } else {
        println!(
            "{}",
            format!("Repository cloned to {}", local.display()).green()
        );
    }

    let open = decide_open(true);
    if open {
        if let Err(e) = git::open_in_vscode(&local, args.dry_run) {
            eprintln!(
                "{}",
                format!("Warning: failed to open VS Code: {}", e).yellow()
            );
        }
    }

    println!(
        "{}",
        format!(
            "To move to the project's directory, please run: \"cd {}\"",
            local.display()
        )
        .cyan()
    );

    // Emit the path for use in shell aliases (e.g., mirror = "!git-mirror && cd $_")
    println!("{}", local.display());
}

#[cfg(test)]
mod tests {
    use super::normalize_args;
    use std::ffi::OsString;

    fn args(values: &[&str]) -> Vec<String> {
        normalize_args(values.iter().map(|v| OsString::from(*v)))
            .into_iter()
            .map(|v| v.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn test_bare_repo_becomes_clone() {
        assert_eq!(
            args(&["git-mirror", "git@github.com:owner/repo.git"]),
            vec!["git-mirror", "clone", "git@github.com:owner/repo.git"]
        );
    }

    #[test]
    fn test_flags_before_repo_become_clone() {
        assert_eq!(
            args(&["git-mirror", "--root", "/tmp", "git@github.com:o/r.git"]),
            vec![
                "git-mirror",
                "clone",
                "--root",
                "/tmp",
                "git@github.com:o/r.git"
            ]
        );
    }

    #[test]
    fn test_explicit_clone_is_untouched() {
        assert_eq!(
            args(&["git-mirror", "clone", "git@github.com:o/r.git"]),
            vec!["git-mirror", "clone", "git@github.com:o/r.git"]
        );
    }

    #[test]
    fn test_migrate_is_untouched() {
        assert_eq!(
            args(&["git-mirror", "migrate", "--apply"]),
            vec!["git-mirror", "migrate", "--apply"]
        );
    }

    #[test]
    fn test_help_and_version_are_untouched() {
        assert_eq!(
            args(&["git-mirror", "--help"]),
            vec!["git-mirror", "--help"]
        );
        assert_eq!(args(&["git-mirror", "-V"]), vec!["git-mirror", "-V"]);
    }

    #[test]
    fn test_no_args_is_untouched() {
        assert_eq!(args(&["git-mirror"]), vec!["git-mirror"]);
    }
}
