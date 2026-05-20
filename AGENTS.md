# Agent Instructions

## Agent skills

### Issue tracker

Issues and PRDs are tracked in self-hosted Forgejo issues for `astrazds/noter-leptos-md`. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default mattpocock/skills triage label vocabulary. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repo: read `CONTEXT.md` at the repo root and ADRs under `docs/adr/` when present. See `docs/agents/domain.md`.

## Rust work on open issues

This repo contains `Cargo.toml`, so agents must apply the `rust-expert` skill whenever working on Rust code. Until all open Forgejo issues are resolved, implementation work should combine:

- `rust-expert` discipline: small idiomatic Rust patches, expressive types where useful, safe Rust by default, and repository toolchain gates.
- `tdd` discipline: red-green-refactor with one behavior-focused test at a time through public Module Interfaces.
- Architecture discipline from the open issues: prefer deep Modules that improve locality and leverage; avoid adding seams unless they earn their keep.

Before finalizing Rust changes, run the relevant cargo checks/tests. If a required target or tool is missing locally, document that explicitly in the final notes instead of treating it as a project failure.
