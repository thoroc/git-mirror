# git-mirror

[![CI](https://github.com/thoroc/git-mirror/actions/workflows/ci.yml/badge.svg)](https://github.com/thoroc/git-mirror/actions/workflows/ci.yml) [![Rust CI](https://github.com/thoroc/git-mirror/actions/workflows/rust.yml/badge.svg)](https://github.com/thoroc/git-mirror/actions/workflows/rust.yml)

Rust CLI to clone a GitHub/GitLab (or other Git) repo to `~/Projects` while keeping a tree
structure close to the remote URL. If the project already exists locally the CLI can
be used to print commands for updating or changing directory instead of cloning.

This repository now contains a Rust implementation (in `src/`) as part of the
`feat/rust-ci-readme-badges` effort.

## Install

Build and install locally with Cargo:

```sh
cargo install --path .
```

This installs a `git-mirror` binary in your Cargo bin directory (usually `~/.cargo/bin`).

Alternatively during development run via:

```sh
cargo run -- <repo> --print-cd
```

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

Basic CLI options (current Rust implementation):

- `-r, --root <rootDir>`  - The root directory (default: `~/Projects`).
  Example: `git-mirror --root ~/Work git@github.com:owner/repo.git`
- `--print-cd`            - Print a shell-friendly `cd` command pointing to the repo local path.
  Example: `git-mirror --print-cd git@github.com:owner/repo.git`
- `--dry-run`             - Print the command that would be run without executing `git clone`.
  Example: `git-mirror --dry-run git@github.com:owner/repo.git`
- `--open-vs-code`        - Open the repo in VS Code after cloning or when it already exists.
  Example: `git-mirror --open-vs-code git@github.com:owner/repo.git`
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

- This repo previously contained a Deno implementation. That TypeScript source has
  been removed on the `feat/rust-ci-readme-badges` branch in favor of the Rust implementation.
- The current Rust CLI is a scaffold and implements core features: host parsing,
  local path construction and `git clone` (with `--dry-run`).
- Next steps: improve edge-case parsing and expand tests.

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

If you want me to publish a release, push the branch, or implement any remaining
features (open in editor, additional flags, or packaging), tell me which and I'll continue.

Developer and contributor information has moved to `CONTRIBUTING.md`.

See `CONTRIBUTING.md` for release automation, contributing, and troubleshooting details.



