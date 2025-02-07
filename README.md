# git-mirror

Deno script to clone a github/gitlab repo to `~/Projects` while keeping a tree structure close to the remote url. If the project is already present, then it'll fetch from the remote.

__WARNING__ this was developed on a MacOS, so no guaranty are offered to run on a different OS.

## install

1. Copy the scripts to local dir (`~/.config/script/git-mirror.ts` for example). Don't forget the `utils.ts` file.
2. add the following alias to your `.gitconfig`:

```toml
[alias]
  mirror = "!Deno run --allow-run --allow-read --allow-env ~/.config/scripts/git-mirror.ts"
```

## Usage

Call `git mirror git@github.com:owner/repo.git` will clone to `~/Projects/owner/repo`.

Options:

* `-o` to open VsCode (default: false)
* `-r` to specify a different root (default: `~/Projects`)
