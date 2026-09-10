# Nota documentation

Start with the guide that matches your task:

| Task | Guide |
| --- | --- |
| Run Nota and understand its current release status | [README](../README.md) |
| Write, search, back up, migrate, or recover Notes | [User guide](usage.md) |
| Set up a development environment or verify a change | [Contributing](../CONTRIBUTING.md) |
| Find the owner of a behavior or data contract | [Architecture](architecture.md) |
| Use the product's domain language | [Context](../CONTEXT.md) |
| Understand the product's users and scope | [Product](../PRODUCT.md) |
| Change native layout, typography, or controls | [Design system](../DESIGN.md) |
| Prepare screenshots, icons, or external copy | [Brand toolkit](brand-toolkit.md) |
| Find the reason for an architectural choice | [Decision records](adr/README.md) |
| Check a packaged app and migration compatibility | [AppImage rehearsal](agents/appimage-rehearsal.md) |
| Understand local storage and network behavior | [Privacy](../PRIVACY.md) |
| Report a vulnerability | [Security](../SECURITY.md) |

Agent workflows use [domain guidance](agents/domain.md), the
[GitHub issue tracker](agents/issue-tracker.md), and the
[triage label vocabulary](agents/triage-labels.md).

Current documentation describes the native-only Cargo workspace. Older ADRs
retain the browser implementation and its rationale as history; their status
notes point to the current owners. Generated API documentation comes from
`mise run doc` and is not edited by hand.
