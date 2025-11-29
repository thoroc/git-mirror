# git-mirror

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

## Usage

Example:

```sh
git-mirror git@github.com:owner/repo.git
```

Basic CLI options (current Rust implementation):

- `-r, --root <rootDir>`  - The root directory (default: `~/Projects`).
- `--print-cd`            - Print a shell-friendly `cd` command pointing to the repo local path.
- `--dry-run`             - Print the command that would be run without executing `git clone`.

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

## Notes & Migration

- This repo previously contained a Deno implementation. That TypeScript source has
  been removed on the `migrate/rust-skeleton` branch in favor of the Rust implementation.
- The current Rust CLI is a scaffold and implements core features: host parsing,
  local path construction and `git clone` (with `--dry-run`).
- Next steps: add `--open-vs-code` support, improve edge-case parsing and expand tests.

---

If you want me to publish a release, push the branch, or implement any remaining
features (open in editor, additional flags, or packaging), tell me which and I'll continue.