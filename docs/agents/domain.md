# Domain documentation for agents

Read [CONTEXT.md](../../CONTEXT.md) before changing domain behavior. Use its
terms for Notes, Tags, Backup, Desktop Transition, and Storage Recovery.

Read additional guidance when the task reaches that area:

- [Architecture](../architecture.md) maps behavior to the core or desktop owner.
- [Product](../../PRODUCT.md) defines users and product scope.
- [Design](../../DESIGN.md) identifies native styling and layout owners.
- [Brand toolkit](../brand-toolkit.md) covers icons, screenshots, and public copy.
- [ADR index](../adr/README.md) distinguishes active decisions from historical
  browser implementation notes.
- [Contributing](../../CONTRIBUTING.md) defines development and verification
  commands through `mise.toml`.

This repository has one domain context. `nota-core` owns toolkit-independent
behavior and compatibility formats. `nota-desktop` owns GTK, WebKitGTK, native
persistence, and the app's effects. The browser frontend is no longer in tree.

When a task exposes missing or ambiguous terminology, record the question and
resolve it in the existing domain document. When a decision changes, add or
amend an ADR with its rationale and link the superseded decision. Preserve the
original context rather than silently rewriting it as if the new decision had
always applied.

See [the documentation index](../README.md) for user and agent entrypoints.
