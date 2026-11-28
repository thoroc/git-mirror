# git-mirror

[![CI](https://github.com/thoroc/git-mirror/actions/workflows/ci.yml/badge.svg)](https://github.com/thoroc/git-mirror/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-0.4.6-blue)](https://github.com/thoroc/git-mirror/releases)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange)](https://www.rust-lang.org/)

Rust CLI to clone a GitHub/GitLab (or other Git) repo to `~/Projects` while keeping a tree
structure close to the remote URL. If the project already exists locally the CLI can
be used to print commands for updating or changing directory instead of cloning.

## Why use git-mirror?

Organizing cloned repositories consistently is tedious. git-mirror automates this by:

- **Smart directory structure** - Mirrors the remote URL structure locally (`~/Projects/github/owner/repo`)
- **Works with any Git host** - GitHub, GitLab, Bitbucket, self-hosted instances, and other forges
- **Migrate existing clones** - Reorganise an existing projects tree from each repository's `origin` remote
- **Shell integration** - Print `cd` commands to jump straight into cloned repos
- **VS Code ready** - Optionally open repos in VS Code immediately after cloning
- **Fast and lightweight** - Written in Rust for maximum performance

## Install

### Quick Install (Recommended)

Install via the automated installation script:

```sh
curl -fsSL https://raw.githubusercontent.com/thoroc/git-mirror/main/install.sh | bash
```

This will:
- Detect your OS and architecture automatically
- Download the latest release binary
- Verify the checksum for security
- Install to `~/.local/bin/git-mirror` (or custom location via `INSTALL_DIR` env var)

Custom installation directory:

```sh
curl -fsSL https://raw.githubusercontent.com/thoroc/git-mirror/main/install.sh | INSTALL_DIR=/usr/local/bin bash
```

### Manual Installation

Download the appropriate binary for your system from the [latest release](https://github.com/thoroc/git-mirror/releases/latest). Archive names include the version tag, e.g. for `v0.4.6`:

- **Linux x86_64**: `git-mirror-linux-x86_64-v0.4.6.tar.gz`
- **Linux ARM64**: `git-mirror-linux-aarch64-v0.4.6.tar.gz`
- **macOS Intel**: `git-mirror-macos-x86_64-v0.4.6.tar.gz`
- **macOS Apple Silicon**: `git-mirror-macos-aarch64-v0.4.6.tar.gz`
- **Windows**: `git-mirror-windows-x86_64-v0.4.6.zip`

Extract and move to a directory in your `PATH`:

```sh
# Linux/macOS example
tar -xzf git-mirror-*.tar.gz
mv git-mirror ~/.local/bin/
chmod +x ~/.local/bin/git-mirror
```

### Build from Source

Build and install locally with Cargo:

```sh
cargo install --path .
```

This installs a `git-mirror` binary in your Cargo bin directory (usually `~/.cargo/bin`).

Alternatively during development run via:

```sh
cargo run -- <repo> --print-cd
```

### Git Alias (Optional)

The quick-install script (above) configures a `git mirror` alias automatically:

```toml
[alias]
  mirror = "!git-mirror \"$@\" && cd \"$_\""
```

Like calling the binary directly, this alias cannot change your parent shell's working directory (see the `cd` caveat under Notes below) — use `--print-cd` with `eval` for that. If you installed manually and want the alias, add a plain version to `~/.gitconfig` yourself:

```toml
[alias]
  mirror = "!git-mirror"
```

After installing, you can call `git-mirror` directly (or `git mirror` if you set the alias).

## Prerequisites

- `git` installed and available on your `PATH`.
- Rust toolchain (`rustc` and `cargo`) if you want to build from source.
- SSH keys configured for SSH-style remotes (if using `git@...` URLs).

## Quick start

Clone using SSH (common):

```sh
git-mirror git@github.com:owner/repo.git
```

Clone using HTTPS:

```sh
git-mirror https://github.com/owner/repo.git
```

Clone from GitLab (example):

```sh
git-mirror git@gitlab.com:group/project.git
```

To just print a `cd` command you can evaluate the output in your shell:

```sh
eval "$(git-mirror git@github.com:owner/repo.git --print-cd)"
```

## Usage

`clone` is the default command, so `git-mirror <repo>` and
`git-mirror clone <repo>` are equivalent.

Basic CLI options:

- `-r, --root <ROOT>`     - Root directory where projects are stored (default: `~/Projects`).
  Example: `git-mirror --root ~/Work git@github.com:owner/repo.git`
- `--print-cd`            - Print a shell-friendly `cd` command pointing to the repo local path.
  Example: `git-mirror --print-cd git@github.com:owner/repo.git`
- `--dry-run`             - Dry run: show commands without executing.
  Example: `git-mirror --dry-run git@github.com:owner/repo.git`
- `--open-vs-code`        - Open the repo in VS Code after cloning or when it already exists.
  Example: `git-mirror --open-vs-code git@github.com:owner/repo.git`
- `--no-open-vs-code`     - Do not open the repo in VS Code.
  Example: `git-mirror --no-open-vs-code git@github.com:owner/repo.git`
- `--no-prompt`           - Disable interactive prompts (useful in CI).
  Example: `git-mirror --no-prompt git@github.com:owner/repo.git`
- `--full-host`           - Use the full host domain in the local path (e.g. `github.com` instead of `github`).
  Example: `git-mirror --full-host git@github.com:owner/repo.git`

Notes:

- The CLI cannot change your parent shell's working directory. To have your interactive
  shell move into the cloned repo automatically, evaluate the CLI output in your shell.

  Bash / Zsh example:

  ```sh
  eval "$(git-mirror git@github.com:owner/repo.git --print-cd)"
  ```

  Fish example:

  ```fish
  eval (git-mirror git@github.com:owner/repo.git --print-cd)
  ```

- If you want only the `cd` (without opening an editor), use `--print-cd`.

## Migrating existing clones

If repositories already live under `~/Projects` in an ad-hoc layout, `git-mirror migrate`
moves each one to the path its `origin` remote implies:

```sh
git-mirror migrate            # dry run: show the plan, change nothing
git-mirror migrate --apply    # perform the moves
```

Options:

- `-r, --root <ROOT>` - Projects root to scan (default: `~/Projects`).
- `--apply` - Perform the moves (default is a dry run).
- `--format text|tsv` - Human-readable output or a TSV table with
  `status`, `current`, `origin`, `target` and `detail` columns. In TSV mode the
  summary goes to stderr, so stdout stays machine-readable.
- `--report <FILE>` - Also write a report to `FILE` (parent directories are created).
  The format is inferred from the extension: `.csv` (default), `.tsv`/`.tab`,
  `.json`, `.md`/`.markdown` or `.txt`/`.text`.
- `--report-format csv|tsv|json|markdown|text` - Override the inferred report
  format. The report is independent of the console `--format`.
- `--include-nested` - Also scan inside repositories for nested repositories.
  This includes vendored clones, so review the dry run before applying.
- `--force` - Move repositories that have linked worktrees (those worktrees will break).
- `--exclude <PREFIX>` - Skip repositories at or under a path prefix (repeatable).
- `--keep <PATH>` - Force the keeper checkout for a duplicate group (repeatable).
- `--duplicates report|consolidate` - Report duplicates (default) or consolidate
  their unique work into the keeper.
- `--archive-dir <DIR>` - Where consolidated sources and patches are kept
  (default: `<root>/.migration-archive`).
- `--full-host` - Use the full host domain in the target path.

Repositories are processed deepest-first. Existing targets are never overwritten:
a target that already exists is reported as a conflict and the source is left in
place. Multiple checkouts of the same remote are scored before one is kept:
unique commits (not reachable from `origin`), uncommitted tracked work (dirty
files plus stashes), in-place location, untracked files, branches, last commit
date and path heuristics. The `detail` column shows those metrics for each
checkout, and `--keep <PATH>` overrides the choice.

With `--duplicates consolidate --apply`, each duplicate's unique branches are
fetched into the keeper as `migration/<YYYY-MM-DD>/<slug>/<branch>`; stashes
become `.../stash-<n>` and a detached HEAD becomes `.../detached-<sha>`. Dirty
tracked changes are written to `worktree.patch`, untracked files are copied under
`untracked/`, and the source checkout is moved (not deleted) to
`<archive-dir>/<date>/<slug>/source`. Refs that are already on `origin` are not
imported, and nested repositories are left in place. Without `--apply`, the same
mode only reports what would be imported and archived.

Typical workflow:

```sh
git-mirror migrate --format tsv > plan.tsv
# review plan.tsv, then:
git-mirror migrate --apply
```

Or keep the human-readable console output and write a machine-readable report
alongside it:

```sh
git-mirror migrate --report plan.csv                         # CSV (default)
git-mirror migrate --report plan.md                          # Markdown table + summary
git-mirror migrate --report plan.json                        # JSON
git-mirror migrate --report plan.tsv --report-format tsv     # explicit override
```

CSV and TSV reports have the columns `status`, `current`, `origin`, `target`
and `detail`. The JSON report wraps the same data with a `summary` object and a
`schema_version`. Markdown writes the same table followed by a summary section.

Example rows (`detail` is empty for plain moves):

```tsv
status	current	origin	target	detail
duplicate	/home/me/Projects/old/error-queue-scheduler-copy	git@bitbucket.org:pplcs/error-queue-scheduler.git	/home/me/Projects/bitbucket/pplcs/error-queue-scheduler	/home/me/Projects/old/error-queue-scheduler
conflict	/home/me/Projects/old/api-gateway	ssh://git@bitbucket.uk.example.com:2222/awstl/api-gateway.git	/home/me/Projects/bitbucket/awstl/api-gateway	 with the same remote
```

### Host mapping

The first path segment is derived from the remote host:

| Remote host | Local directory |
| --- | --- |
| `github.com`, `*.github.com` | `github` |
| `gitlab.com`, `*.gitlab.com` | `gitlab` |
| `bitbucket.org`, `*.bitbucket.org` | `bitbucket` |
| any host containing `bitbucket` | `bitbucket` |
| any host containing `gitlab` | `gitlab` |
| any host containing `github` | `github` |
| `go.googlesource.com` | `googlesource` |
| `gopkg.in` | `gopkg` |
| `phabricator.*` | `phabricator` |
| anything else | first DNS label (`git.example.com` -> `git`) |

Use `--full-host` to keep the full domain instead, for example
`~/Projects/bitbucket.uk.example.com/qa/repo`.

## Shell helpers

Copy-paste helper functions for your shell.

Bash / Zsh:

```sh
git_mirror_cd() {
  eval "$(git-mirror "$1" --print-cd ${2:+--root "$2"})"
}
```

Fish:

```fish
function git_mirror_cd
  set -l repo $argv[1]
  set -l root_arg ''
  if test (count $argv) -ge 2
    set root_arg "--root $argv[2]"
  end
  eval (git-mirror $repo --print-cd $root_arg)
end
```

PowerShell:

```powershell
function Git-Mirror-Cd {
  param([string]$Repo, [string]$Root)
  if ($Root) {
    $cmd = git-mirror $Repo --print-cd --root "$Root"
  } else {
    $cmd = git-mirror $Repo --print-cd
  }
  Invoke-Expression $cmd
}
```

## Notes & Migration

- This repo previously contained a Deno/TypeScript implementation, which has been replaced with the current Rust implementation.
- The Rust CLI implements host parsing, local path construction, `git clone`, migration, VS Code integration, and interactive prompts.

### Migration notes (host-name change)

The Rust implementation uses a short host label by default when constructing the
local path. For example:

- `git@github.com:owner/repo.git` -> `~/Projects/github/owner/repo`

If you prefer the full domain (e.g. `github.com`) in the local path, use the
`--full-host` flag:

```sh
git-mirror --full-host git@github.com:owner/repo.git
# => ~/Projects/github.com/owner/repo
```

If you already have clones in a different layout, use
[`git-mirror migrate`](#migrating-existing-clones) to move them into place
instead of re-cloning them.

---

Developer and contributor information has moved to `CONTRIBUTING.md`.

See `CONTRIBUTING.md` for release automation, contributing, and troubleshooting details.



