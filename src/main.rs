mod fs;
mod git;
mod util;

use clap::Parser;
use dialoguer::Confirm;
use owo_colors::OwoColorize;
use std::env;
use std::process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
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

fn main() {
    let cli = Cli::parse();

    let local = match fs::build_local_repo_path(&cli.root, &cli.repo, cli.full_host) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", format!("Error building local path: {}", e).red());
            process::exit(1);
        }
    };

    // If the user only wants the cd command, print it and exit early
    if cli.print_cd {
        println!("{}", format!("cd \"{}\"", local.display()).green());
        return;
    }

    // Decide whether to open VS Code: explicit flags take precedence,
    // otherwise prompt the user. If running in CI or `--no-prompt` is set, do not prompt.
    let is_ci = env::var("CI").is_ok();
    let decide_open = |default: bool| -> bool {
        // Always open if explicitly requested
        if cli.open_vs_code {
            return true;
        }

        // Never open if explicitly disabled
        if cli.no_open_vs_code {
            return false;
        }

        // Never open in CI environments
        if is_ci {
            return false;
        }

        // Use default without prompting if --no-prompt is set
        if cli.no_prompt {
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
        if cli.dry_run {
            println!(
                "{}",
                format!("> Dry run: Fetching repository: {}", local.display()).yellow()
            );
        } else {
            if let Err(e) = git::fetch_repo(&local, cli.dry_run) {
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
            if let Err(e) = git::open_in_vscode(&local, cli.dry_run) {
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
    if let Err(e) = git::clone_repo(&cli.repo, &local, cli.dry_run) {
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
        if let Err(e) = git::open_in_vscode(&local, cli.dry_run) {
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
