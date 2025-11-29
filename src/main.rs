mod util;

use clap::Parser;

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

    match util::get_host_from_repo(&cli.repo) {
        Ok(host) => println!("Host: {}", host),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
