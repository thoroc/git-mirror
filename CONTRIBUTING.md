Contributing
============

Thank you for considering contributing to git-mirror! This document contains developer and contributor information moved out of the main README.

Quick notes
-----------

- The Rust implementation lives in `src/`. Contributions (bug fixes, parsing improvements, tests) are welcome.
- To run locally during development: `cargo run -- <repo> --print-cd`.
- Consider adding unit tests for parsing edge cases; run tests with `cargo test`.

Setting up release automation
-----------------------------

- Create a GitHub Personal Access Token (PAT) with `repo` scope (or `public_repo` for public repositories).
- In your repository settings, go to `Settings > Secrets and variables > Actions` and create a new secret named `RELEASE_PLEASE_TOKEN` with the PAT value.
- The workflow `.github/workflows/release-please.yml` uses this secret to allow `release-please` to create PRs and Releases that trigger CI checks.

Note: rotate the PAT periodically and limit its permissions to only what's necessary.

Fine-grained token permissions (recommended)
--------------------------------------------

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
--------------------------------

- Use the narrowest scope possible and limit the token to this repository only.
- Prefer fine-grained tokens over classic PATs when available.
- Rotate the token regularly and remove it if the user account is removed or compromised.
- If you need CI runs on release-please PRs and releases, the PAT must be from a user with write access to the repository.

Troubleshooting
---------------

- Permission denied when cloning via SSH: ensure your SSH key is added to your GitHub/GitLab account and `ssh-agent` is running.
- Remote not found / invalid URL: check the remote URL format (SSH vs HTTPS) and ensure it is correct.
- Path collisions when using `--full-host`: double-check existing directories under the root and consider `--full-host` only for preserving old layouts.
- `git` not found: install `git` and ensure it's on your `PATH`.
