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

PR review automation
---------------------

- `.github/workflows/pr-agent.yml` posts an LLM-generated description and review on pull requests via [PR-Agent](https://github.com/qodo-ai/pr-agent). It is advisory only (`continue-on-error: true`) and never blocks a merge.
- It only runs if at least one provider key is configured as a repository secret: `GEMINI_API_KEY` (primary), `MISTRAL_API_KEY`, or `CEREBRAS_API_KEY` (fallbacks). With none set, the workflow skips cleanly and posts a notice.
- Add secrets under `Settings > Secrets and variables > Actions`. Remove all three to disable the review entirely.

CI/CD security scanning (Plumber)
---------------------------------

- [Plumber](https://github.com/getplumber/plumber) (`.github/workflows/plumber.yml`) scans this repository's own GitHub Actions workflows and configuration for issues like unpinned actions, untrusted shell input, missing branch protection, and over-broad permissions. Config lives in `.plumber.yaml` (extends Plumber's default control set). Runs on push to `main`, on PRs, and weekly.
- Findings are graded Critical/High/Medium/Low. Only **Critical** blocks the PR (`scripts/plumber-gate.sh`); High/Medium/Low are tracked but never blocking — they land in a rolling backlog issue on push to `main` (`scripts/plumber-file-issues.sh`) and a per-PR comment (`scripts/plumber-pr-comment.sh`). All findings also upload to the Security tab as SARIF.
- The Plumber action itself runs with `continue-on-error: true`, so its own score gate or a transient runtime error never blocks a merge — the only blocking signal is a Critical finding via `plumber-gate.sh`.

Secret scanning
----------------

- [Kingfisher](https://github.com/mongodb/kingfisher) scans for leaked credentials in two places: a pre-commit hook (`kingfisher scan . --staged`, only the staged diff) and `.github/workflows/kingfisher.yml` (full working tree + git history, on push to `main`, on PRs, and weekly). `mise install` installs the binary and wires up the hook automatically.
- Unlike Plumber's High/Medium/Low backlog model, any new finding here fails the check — a leaked secret is always-blocking, not tracked-and-deferred. Findings also upload to the Security tab as SARIF.
- `.kingfisher-baseline.yml` (repo root) suppresses known non-issues so the scan can actually pass — currently a documentation example and some test-fixture strings baked into a `target/` build artifact that was briefly committed to history. It is not a place to hide a real secret.
- If the scan flags something new:
  - **A real secret**: rotate/revoke it immediately, then remove it from the code. Don't baseline it.
  - **A false positive**: regenerate the baseline with `kingfisher scan . --manage-baseline --baseline-file .kingfisher-baseline.yml` (run from the repo root so paths match) and commit the updated file.

Dependency updates
------------------

- [Renovate](https://github.com/renovatebot/renovate) is configured via `renovate.json5` at the repo root. It covers three ecosystems: Cargo (`Cargo.toml`/`Cargo.lock`), GitHub Actions (including keeping SHA-pinned actions' digests current), and mise (`mise.toml`/`mise.lock` — every tool: `rust`, `hk`, `pkl`, `shellcheck`, `cargo-audit`, `cargo-tarpaulin`, `kingfisher`).
- All non-major updates (minor, patch, pin, digest) land in a single weekly PR across every ecosystem, to keep review overhead low. Major updates each get their own independent PR.
- Renovate only looks for updates once a week: Monday, 00:00-03:59 UTC.
- **The Renovate GitHub App still needs to be installed on this repository** — the config file alone doesn't activate it. Install it at <https://github.com/apps/renovate> (or run it self-hosted via a scheduled GitHub Actions workflow) and grant it access to this repo.

Troubleshooting
---------------

- Permission denied when cloning via SSH: ensure your SSH key is added to your GitHub/GitLab account and `ssh-agent` is running.
- Remote not found / invalid URL: check the remote URL format (SSH vs HTTPS) and ensure it is correct.
- Path collisions when using `--full-host`: double-check existing directories under the root and consider `--full-host` only for preserving old layouts.
- `git` not found: install `git` and ensure it's on your `PATH`.
