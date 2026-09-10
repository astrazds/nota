# Architecture decision records

ADRs record decisions and their original context. Browser-era implementation
notes remain historical evidence, not instructions for the current checkout.
Use [the architecture guide](../architecture.md) for current module ownership
and [CONTRIBUTING.md](../../CONTRIBUTING.md) for commands.

| Record | Current status |
| --- | --- |
| [0001: Vertical slices](0001-redesign-in-vertical-slices.md) | The delivery principle still applies. The initial redesign sequence is complete. |
| [0002: Merge-only Backup import](0002-merge-only-backup-import.md) | Active. Native Merge Import also recovers matching Recently Deleted identities. |
| [0003: Browser audit polish](0003-browser-audit-polish.md) | Historical browser implementation. Product behavior carries into native Nota. |
| [0004: Capture and recovery](0004-fast-capture-and-local-recovery.md) | Product rules remain active. Native persistence replaces the LocalStorage implementation. |
| [0005: Editor controls and Clear All](0005-editor-first-controls-and-clear-all.md) | Product rules remain active. GTK replaces the browser implementation. |
| [0006: Core modules and browser tests](0006-deep-core-modules-and-browser-visual-contracts.md) | Domain ownership principles remain. Browser modules and Playwright tooling were removed. |
| [0007: Visual system](0007-local-notebook-visual-system.md) | Visual intent remains. Native CSS, Pango, and packaged fonts replace Tailwind and Trunk. |
| [0008: Browser storage for 1.0](0008-static-browser-storage-for-1-0.md) | Superseded for the native product by ADR-0009. Describes the browser migration source. |
| [0009: Native replacement](0009-relm4-native-replacement.md) | Implemented. Its source-cutover section records completion; format compatibility remains active. |
| [0010: AppImage distribution](0010-appimage-first-native-distribution.md) | Active packaging decision. The original manual transfer gate passed; repeat the rehearsal for relevant changes. |
| [0011: Nota identifiers](0011-product-name-nota.md) | Active. Legacy identifiers remain accepted for migration. |

Source cutover, a stable release, and hosted browser retirement are separate
milestones. Completion of a local rehearsal does not itself publish or retire
anything.
