# git-mirror

Deno script to clone a github/gitlab repo to `~/Projects` while keeping a tree structure close to the remote url. If the project is already present, then it'll fetch from the remote.

__WARNING__ this was developed on a MacOS, so no guaranty are offered to run on a different OS.

## install

1. Clone the repo to a sensible place (eg: `~/.git-mirror`)
2. Add the following alias to your `.gitconfig`:

```toml
[alias]
  mirror = "!Deno run --allow-run --allow-read --allow-env ~/.git-mirror/git-mirror.ts"
```

## Usage

Call `git mirror git@github.com:owner/repo.git` will clone to `~/Projects/owner/repo`.

```sh
Usage:   clone <repo>
Version: 0.1.1       

Description:

  Clone a Git repository into the ~/Projects directory.

Options:

  -h, --help                     - Show this help.                                                               
  -V, --version                  - Show the version number for this program.                                     
  -r, --root          <rootDir>  - The root directory.                        (Default: "/Users/<user>/Projects")
  -o, --open-vs-code             - Open the repository in VS Code.            (Default: false)                   
```

Options:

* `-o` to open VsCode (default: false)
* `-r` to specify a different root (default: `~/Projects`)
