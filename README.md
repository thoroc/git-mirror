# git-mirror

Deno script to clone a github/gitlab repo to `~/Projects` while keeping a tree
structure close to the remote url. If the project is already present, then it'll
fetch from the remote.

**WARNING** this was developed on a MacOS, so no guaranty are offered to run on
a different OS.

## install

1. Clone the repo to a sensible place (eg: `git clone ~/.git-mirror`).
   Alternatively grab the latests
   [release](https://github.com/thoroc/git-mirror/releases/tag/v0.1.10)
2. Add the following alias to your `.gitconfig`:

```toml
[alias]
  mirror = "!Deno run --allow-run --allow-read --allow-env ~/.git-mirror/git-mirror.ts"
```

## Usage

Call `git mirror git@github.com:owner/repo.git` will clone to
`~/Projects/owner/repo`.

```sh
Usage:   git-mirror <repo>
Version: 0.1.10

Description:

  Clone a Git repository into the ~/Projects directory.

Options:

  -h, --help                     - Show this help.                                                               
  -V, --version                  - Show the version number for this program.                                      
  -r, --root          <rootDir>  - The root directory.                        (Default: "/Users/<user>/Projects")
  -o, --open-vs-code             - Open the repository in VS Code.            (Default: true)                    
  --no-open-vs-code              - Do not open the repository in VS Code.                                        
  --dry-run                      - Print the command that would be run.
  --print-cd                     - Print a shell-friendly command that opens VS Code (if enabled) and then cds into the repo.
```

Notes and examples:

- A program cannot change its parent shell's working directory. To have your interactive shell move into the cloned repo automatically, evaluate the CLI output in your shell.

  - Bash / Zsh example:

    eval "$(git-mirror git@github.com:owner/repo.git --print-cd)"

  - Fish example:

    eval (git-mirror git@github.com:owner/repo.git --print-cd)

- If you want only the `cd` (without opening VS Code), pass `--print-cd --no-open-vs-code` (or explicitly set the `--open-vs-code` flag as desired).

- You can create a shell helper function to wrap this behavior. See the separate "Shell helpers" section below for copy-paste snippets.


## Shell helpers

Below are copy-paste helper functions you can add to your shell rc file to run `git-mirror` and automatically change into the cloned repository.

- Bash (add to `~/.bashrc` or `~/.bash_profile`):

```sh
git_mirror_cd() {
  eval "$(git-mirror "$1" --print-cd ${2:+--root "$2"})"
}
```

- Zsh (add to `~/.zshrc`):

```sh
git_mirror_cd() {
  eval "$(git-mirror "$1" --print-cd ${2:+--root "$2"})"
}
```

- Fish (add to `~/.config/fish/config.fish`):

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

You can quickly append a helper to your rc file using a one-liner.

- Append the Bash/Zsh helper to `~/.bashrc` or `~/.zshrc`:

```sh
echo 'git_mirror_cd() { eval "$(git-mirror "$1" --print-cd ${2:+--root "$2"})"; }' >> ~/.bashrc
# or for zsh
echo 'git_mirror_cd() { eval "$(git-mirror "$1" --print-cd ${2:+--root "$2"})"; }' >> ~/.zshrc
```

- Append the Fish helper to your Fish config:

```sh
cat >> ~/.config/fish/config.fish <<'FISH'
function git_mirror_cd
  set -l repo $argv[1]
  set -l root_arg ''
  if test (count $argv) -ge 2
    set root_arg "--root $argv[2]"
  end
  eval (git-mirror $repo --print-cd $root_arg)
end
FISH
```

After adding the helper, `source` the rc file or open a new shell for it to take effect.

Then call `git_mirror_cd git@github.com:owner/repo.git` to clone and change directory automatically.

