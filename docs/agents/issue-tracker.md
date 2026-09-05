# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues in `astrazds/nota`. Use the
`gh` CLI for all operations.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."`. For long
  bodies, write the body to a temporary file and pass `--body-file <path>`.
- **Read an issue body**: `gh issue view <number>`
- **Read issue comments**: `gh issue view <number> --comments`
- **Search/list issues**: `gh issue list --state open` with `--label`,
  `--assignee`, `--author`, and `--search` filters.
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply / remove labels**: `gh issue edit <number> --add-label "..."` /
  `--remove-label "..."`
- **Close**: `gh issue close <number> --comment "..."`

Infer the repo from `git remote -v`. `gh` accepts `-R <owner/repo>` when the
target repo needs to be specified explicitly.

## When a skill says "publish to the issue tracker"

Create a GitHub issue with `gh issue create`. Latest mattpocock skills name
this `to-spec` (was `to-prd`) and `to-tickets` (was `to-issues`). For
`to-tickets` blocking edges, use `gh issue create --blocked-by <n>` or
`gh issue edit <n> --add-blocked-by <blocker>`.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number>` and `gh issue view <number> --comments`.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a single issue with **child** issues as
tickets.

- **Map**: a single issue labelled `wayfinder:map`, holding the Notes /
  Decisions-so-far / Fog body. `gh issue create --title "..." --body "..."`
  then `gh issue edit <n> --add-label wayfinder:map`.
- **Child ticket**: `gh issue create --parent <map> --label wayfinder:<type>`
  (`research` / `prototype` / `grilling` / `task`) with `Part of #<map>` at the
  top of its body. Once claimed, the ticket is assigned to the driving dev.
- **Blocking**: GitHub issue blocking relationships. Add a blocker with
  `gh issue edit <child> --add-blocked-by <blocker>` (`<child>` is blocked by
  `<blocker>`). A ticket is unblocked when every blocker is closed.
- **Frontier query**: `gh issue list --state open --json number,title,body,labels,assignees,blockedBy,parent`
  scoped to the map's children (`parent.number` equals the map, or the body
  starts with `Part of #<map>`), drop any with an open blocker or an assignee;
  first in map order wins.
- **Claim**: `gh issue edit <n> --add-assignee @me`, the session's first write.
- **Resolve**: `gh issue comment <n> --body "..."`, then
  `gh issue close <n> --comment "..."`, then append a context pointer (gist +
  link) to the map's Decisions-so-far.
