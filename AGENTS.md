# AGENTS.md

Domain terms and ownership live in [docs/agents/domain.md](docs/agents/domain.md).
Local notes stay in `AGENTS.local.md`, which is gitignored.

## Rules these checks enforce

| Rule | What enforces it |
| --- | --- |
| A pull request body does not contain the footer comment `<!-- CURSOR_AGENT_PR_BODY`. Remove the footer. | `node scripts/check-pr-body.mjs` in `.github/workflows/pr-body.yml`, with `PR_BODY` set |
| A commit in the pull request does not contain a `Co-authored-by` trailer for `cursoragent@cursor.com`. Commit as Andrejs Strazds and remove the Cursor agent trailer. | `node scripts/check-pr-body.mjs` in `.github/workflows/pr-body.yml`, with `BASE_SHA` set to the base commit |
