# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues in `astrazds/nota`. Use the
`gh` CLI for all operations and pass `--repo astrazds/nota` explicitly.
Use each command's `--help` output to check relationship flags when your
installed CLI version differs.

## Conventions

- **Create an issue**: `gh issue create --repo astrazds/nota --title "..." --body "..."`. For long
  bodies, write the body to a temporary file and pass `--body-file <path>`.
- **Read an issue body**: `gh issue view --repo astrazds/nota <number>`
- **Read issue comments**: `gh issue view --repo astrazds/nota <number> --comments`
- **Search/list issues**: `gh issue list --repo astrazds/nota --state open` with `--label`,
  `--assignee`, `--author`, and `--search` filters.
- **Comment on an issue**: `gh issue comment --repo astrazds/nota <number> --body "..."`
- **Apply / remove labels**: `gh issue edit --repo astrazds/nota <number> --add-label "..."` /
  `--remove-label "..."`
- **Close**: `gh issue close --repo astrazds/nota <number> --comment "..."`

This checkout can have several remotes. Use the canonical GitHub repository
`astrazds/nota`; `gh` accepts `--repo astrazds/nota` or `-R astrazds/nota`.
Keep the explicit repository option in commands copied from this guide.

## When a skill says "publish to the issue tracker"

Create a GitHub issue with `gh issue create --repo astrazds/nota`. The
`to-spec` and `to-tickets` skills use this tracker. For
`to-tickets` blocking edges, use `gh issue create --repo astrazds/nota --blocked-by <n>` or
`gh issue edit --repo astrazds/nota <n> --add-blocked-by <blocker>`.

## When a skill says "fetch the relevant ticket"

Run `gh issue view --repo astrazds/nota <number>` and `gh issue view --repo astrazds/nota <number> --comments`.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a single issue with **child** issues as
tickets.

- **Map**: a single issue labelled `wayfinder:map`, holding the Notes /
  Decisions-so-far / Fog body. `gh issue create --repo astrazds/nota --title "..." --body "..."`
  then `gh issue edit --repo astrazds/nota <n> --add-label wayfinder:map`.
- **Child ticket**: `gh issue create --repo astrazds/nota --parent <map> --label wayfinder:<type>`
  (`research` / `prototype` / `grilling` / `task`) with `Part of #<map>` at the
  top of its body. Once claimed, the ticket is assigned to the driving dev.
- **Blocking**: GitHub issue blocking relationships. Add a blocker with
  `gh issue edit --repo astrazds/nota <child> --add-blocked-by <blocker>` (`<child>` is blocked by
  `<blocker>`). A ticket is unblocked when every blocker is closed.
- **Frontier query**: `gh issue list --repo astrazds/nota --state open --json number,title,body,labels,assignees,blockedBy,parent`
  scoped to the map's children (`parent.number` equals the map, or the body
  starts with `Part of #<map>`), drop any with an open blocker or an assignee;
  first in map order wins.
- **Claim**: `gh issue edit --repo astrazds/nota <n> --add-assignee @me`, the session's first write.
- **Resolve**: `gh issue comment --repo astrazds/nota <n> --body "..."`, then
  `gh issue close --repo astrazds/nota <n> --comment "..."`, then append a context pointer (gist +
  link) to the map's Decisions-so-far.
