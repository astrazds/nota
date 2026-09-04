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

Create a Forgejo issue with `fj issue create`. Latest mattpocock skills name this `to-spec` (was `to-prd`) and `to-tickets` (was `to-issues`). For `to-tickets` blocking edges, use native dependencies (`fj issue dependencies add <child> <blocker>`); fall back to a `Blocked by: #<n>` line at the top of the child body if the dependency API is unavailable.

## When a skill says "fetch the relevant ticket"

Run `fj issue view <number>` and `fj issue view <number> comments`.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a single issue with **child** issues as tickets.

- **Map**: a single issue labelled `wayfinder:map`, holding the Notes / Decisions-so-far / Fog body. `fj issue create "..." --body "..."` then apply `wayfinder:map`.
- **Child ticket**: an issue carrying `Part of #<map>` at the top of its body and labels `wayfinder:<type>` (`research` / `prototype` / `grilling` / `task`). Once claimed, the ticket is assigned to the driving dev.
- **Blocking**: Forgejo's **native issue dependencies**, the canonical, UI-visible representation. Add a blocker with `fj issue dependencies add <child> <blocker>` (`<child>` is blocked by `<blocker>`). Where the dependency API is unavailable, fall back to a `Blocked by: #<n>, #<n>` line at the top of the child body. A ticket is unblocked when every blocker is closed.
- **Frontier query**: `fj issue search --state open` scoped to the map's children (issues whose body starts with `Part of #<map>`), drop any with an open blocker (`fj issue dependencies list <n>`) or an assignee; first in map order wins.
- **Claim**: `fj issue edit <n> --assignee <you>`, the session's first write.
- **Resolve**: `fj issue comment <n> "..."`, then `fj issue close <n> --with-msg "..."`, then append a context pointer (gist + link) to the map's Decisions-so-far.
