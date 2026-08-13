# Agent Instructions for `charming-testkit`

> [!IMPORTANT]
> `charming-testkit` is an ORIGINAL crate (the PTY-driven integration test harness for the
> `charming-*` TUI crates) — it is not a port of any upstream Go repository, so the
> upstream-mirror version policy does NOT apply to it. `scripts/verify_upstream_version.sh`
> treats it accordingly.

## Core Rules & Workflow
1. Refer to the workspace-level rules in [`../AGENTS.md`](../AGENTS.md).
2. Maintain 100% rustdoc documentation.
3. Verify all tests pass with `cargo test --all-targets` before committing.

## Releases
- This crate has no upstream to mirror; version numbers are free (current: `0.1.x`).
- To release: push the `v*` tag (e.g. `git tag v0.1.1 && git push origin v0.1.1`); the
  workflow runs tests, creates the GitHub Release, and attempts the crates.io publish
  (non-fatal without a registry token).
- The crates.io publish step is tag-gated; dev pushes only run tests.
