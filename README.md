# git-mirror

[![CI](https://github.com/thoroc/git-mirror/actions/workflows/ci.yml/badge.svg)](https://github.com/thoroc/git-mirror/actions/workflows/ci.yml)

Rust CLI to clone a GitHub/GitLab (or other Git) repo to `~/Projects` while keeping a tree
structure close to the remote URL. If the project already exists locally the CLI can
be used to print commands for updating or changing directory instead of cloning.

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

Download the appropriate binary for your system from the [latest release](https://github.com/thoroc/git-mirror/releases/latest):

- **Linux x86_64**: `git-mirror-linux-x86_64.tar.gz`
- **Linux ARM64**: `git-mirror-linux-aarch64.tar.gz`
- **macOS Intel**: `git-mirror-macos-x86_64.tar.gz`
- **macOS Apple Silicon**: `git-mirror-macos-aarch64.tar.gz`
- **Windows**: `git-mirror-windows-x86_64.zip`

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

To add a Git alias for convenience, add to your `~/.gitconfig`:

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
- The Rust CLI implements core features: host parsing, local path construction, `git clone`, VS Code integration, and interactive prompts.

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

If you are migrating from an earlier implementation that used the full domain
in local paths, consider running a one-time move of your directories or cloning
with `--full-host` to keep the old layout.

If you want me to add an automatic migration helper (move directories and create
symlinks), say so and I can implement it.

---

Developer and contributor information has moved to `CONTRIBUTING.md`.

See `CONTRIBUTING.md` for release automation, contributing, and troubleshooting details.



