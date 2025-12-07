Workflows
=========

This repository uses GitHub Actions workflows located in `.github/workflows/`.

Active workflows
----------------
- `ci.yml` - Consolidated CI for formatting, linting (clippy) and tests. Runs on `ubuntu-latest` and `macos-latest`.
- `release-please.yml` - Release automation using `release-please` to create release PRs and releases.

Deprecated / disabled workflows
------------------------------
- `deploy.yml` - Previously a Deno-based deploy pipeline; now disabled and kept for reference.
- `rust.yml` - Deprecated; consolidated into `ci.yml`.

Recommendations
---------------
- Keep `ci.yml` as the authoritative CI file and remove duplicates.
- Use least-privilege permissions in workflows; only grant `contents: write` or `pull-requests: write` where strictly required.
- Add caching steps for Cargo (`~/.cargo/registry`, `~/.cargo/git`, `target`) to speed CI runs.
- Avoid workflows that modify or commit repository files automatically; prefer release actions that create PRs or manual steps.
