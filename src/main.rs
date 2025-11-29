mod util;
mod git;
mod fs;

use clap::Parser;
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
}

fn main() {
    let cli = Cli::parse();

    let local = match fs::build_local_repo_path(&cli.root, &cli.repo) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error building local path: {}", e);
            process::exit(1);
        }
    };

    if cli.print_cd {
        println!("cd \"{}\"", local.display());
        return;
    }

    if let Err(e) = git::clone_repo(&cli.repo, &local, cli.dry_run) {
        eprintln!("Error cloning repo: {}", e);
        process::exit(1);
    }
}
