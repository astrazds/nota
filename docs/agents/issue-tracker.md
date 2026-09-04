# Issue tracker: Forgejo

Issues and PRDs for this repo live as self-hosted Forgejo issues in `astrazds/noter-leptos-md`. Use the `fj` CLI for all operations.

## Conventions

- **Create an issue**: `fj issue create "..." --body "..."`. For long bodies, write the body to a temporary file and pass `--body-file <path>`.
- **Read an issue body**: `fj issue view <number>`
- **Read issue comments**: `fj issue view <number> comments`
- **Search/list issues**: `fj issue search --state open` with appropriate `--labels`, `--creator`, `--assignee`, and `--state` filters.
- **Comment on an issue**: `fj issue comment <number> "..."`
- **Apply / remove labels**: `fj issue edit <number> labels --add "..."` / `--rm "..."`
- **Close**: `fj issue close <number> --with-msg "..."`

Infer the repo from `git remote -v`. `fj issue` accepts `--remote <remote>` if the target remote needs to be specified explicitly, and `--repo <owner/repo>` for commands that support it.

## When a skill says "publish to the issue tracker"

Create a Forgejo issue with `fj issue create`. Latest mattpocock skills name this `to-spec` (was `to-prd`) and `to-tickets` (was `to-issues`).

## When a skill says "fetch the relevant ticket"

Run `fj issue view <number>` and `fj issue view <number> comments`.
