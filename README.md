# git-mirror

[![CI](https://github.com/thoroc/git-mirror/actions/workflows/ci.yml/badge.svg)](https://github.com/thoroc/git-mirror/actions/workflows/ci.yml) [![Rust CI](https://github.com/thoroc/git-mirror/actions/workflows/rust.yml/badge.svg)](https://github.com/thoroc/git-mirror/actions/workflows/rust.yml)

Rust CLI to clone a GitHub/GitLab (or other Git) repo to `~/Projects` while keeping a tree
structure close to the remote URL. If the project already exists locally the CLI can
be used to print commands for updating or changing directory instead of cloning.

This repository now contains a Rust implementation (in `src/`) as part of the
`migrate/rust-skeleton` effort.

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
  been removed on the `migrate/rust-skeleton` branch in favor of the Rust implementation.
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

**Setting up release automation**

- Create a GitHub Personal Access Token (PAT) with `repo` scope (or `public_repo` for public repositories).
- In your repository settings, go to `Settings > Secrets and variables > Actions` and create a new secret named `RELEASE_PLEASE_TOKEN` with the PAT value.
- The workflow ` .github/workflows/release-please.yml ` uses this secret to allow `release-please` to create PRs and Releases that trigger CI checks.

Note: rotate the PAT periodically and limit its permissions to only what's necessary.

Fine-grained token permissions (recommended)

- Create a **Fine‑grained personal access token** instead of a classic PAT and restrict it to this repository.
- Minimum repository permissions to set when creating the token:
  - `Contents` → Read & write
  - `Pull requests` → Read & write
  - `Metadata` → Read-only
- Optional permissions (enable only if needed):
  - `Workflows` → Read-only (or Read & write if you need the token to re-run or manage workflow runs)

Step-by-step: create a fine-grained token and add the secret

1. Go to GitHub → your avatar → **Settings** → **Developer settings** → **Personal access tokens** → **Fine-grained tokens** → **Generate new token**.
2. Under **Resource owner** choose your account.
3. Under **Repositories**, select **Only select repositories** and pick this repository.
4. Under **Permissions**, set the permissions listed above.
5. Set an expiration (choose a short lifecycle if possible) and create the token.
6. Copy the token value once (you won't be able to see it later).
7. In the repository, go to **Settings → Secrets and variables → Actions → New repository secret** and add a secret:
   - Name: `RELEASE_PLEASE_TOKEN`
   - Value: the token you copied

Notes & security recommendations

- Use the narrowest scope possible and limit the token to this repository only.
- Prefer fine-grained tokens over classic PATs when available.
- Rotate the token regularly and remove it if the user account is removed or compromised.
- If you need CI runs on release-please PRs and releases, the PAT must be from a user with write access to the repository.

Troubleshooting

- Permission denied when cloning via SSH: ensure your SSH key is added to your GitHub/GitLab account and `ssh-agent` is running.
- Remote not found / invalid URL: check the remote URL format (SSH vs HTTPS) and ensure it is correct.
- Path collisions when using `--full-host`: double-check existing directories under the root and consider `--full-host` only for preserving old layouts.
- `git` not found: install `git` and ensure it's on your `PATH`.

Contributing & Tests

- The Rust implementation lives in `src/`. Contributions (bug fixes, parsing improvements, tests) are welcome.
- To run locally during development: `cargo run -- <repo> --print-cd`.
- Consider adding unit tests for parsing edge cases; run tests with `cargo test`.

