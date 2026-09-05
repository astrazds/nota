# Static browser storage for 1.0

For the 1.0 milestone, Nota remains a static browser app with local browser storage, explicit Backup, Recently Deleted recovery, and user-visible trust improvements around the existing Flat Collection model. SQLite, IndexedDB record-store migration, desktop shells, local services, sync, and server backends are deferred until after 1.0 because a storage-engine migration would introduce the highest data-risk change in the part of the app users most need to trust.

Accepted decisions:

- Treat 1.0 as a daily-use trust milestone for the current local-first browser app rather than a storage-platform migration.
- Keep LocalStorage as the 1.0 persistence engine for active Notes, Recently Deleted Notes, Theme preference, Responsive Navigation preference, and Backup Health metadata.
- Harden the current storage story with user-visible corrupt saved-data recovery, Backup Health nudges, documented release checks, and Backup export/import coverage before considering a new storage engine.
- Add a minimal previous-snapshot safety net before overwriting the active Notes and Recently Deleted LocalStorage entries, so corrupt startup can offer a last known-good restore path alongside starting empty or importing a Backup.
- On corrupt startup, do not automatically restore the previous snapshot. Show a visible recovery state and let the user choose whether to restore the previous snapshot, start empty, or import a Backup.
- Update the previous snapshot only after the next active Notes and Recently Deleted payloads have been serialised and validated, immediately before overwriting the current LocalStorage entries.
- Restoring the previous snapshot should exactly replace active Notes and Recently Deleted with the last known-good pair; it should not merge with the corrupt or partially loaded current storage.
- Present corrupt startup recovery as a main app display state, not as a blocking modal before the app loads.
- When the user chooses to start empty after corrupt startup, preserve the corrupt raw active Notes and Recently Deleted payloads under diagnostic quarantine keys before overwriting current storage with an empty collection.
- Show missing or stale Backup Health as an explicit but quiet call to action in the Backup footer, keeping Export immediately available without adding persistent warning chrome.
- Keep the existing 14-day stale Backup Health threshold for 1.0.
- Add a small secondary diagnostics surface for app version, storage mode, Backup Health, and whether corrupt payloads are quarantined.
- Keep Backup as the explicit local recovery mechanism for 1.0; do not introduce sync semantics or a server backend.
- Defer installable PWA and offline service-worker work unless it proves nearly free, because it creates a separate trust surface around asset caching and update behavior.
- Keep the full existing Rust, build, and browser verification gate for 1.0, and add one focused recovery smoke path for corrupt storage, previous snapshot restore, start empty, and Backup import availability.
- Defer SQLite to a post-1.0 ADR if Nota moves toward a desktop shell, local service, or browser SQLite/OPFS direction.

Considered options:

- SQLite backend: stronger transactions, migrations, scaling, and future full-text search, but it implies a desktop shell, local service, server backend, or browser SQLite/OPFS complexity.
- IndexedDB record store: a more browser-native post-1.0 migration path, but still carries migration and test-matrix risk.
- Current LocalStorage plus hardening: lower architectural change for 1.0 and keeps the product aligned with the existing local-first Backup model.

Consequences:

- 1.0 work should focus on data confidence, recovery visibility, release documentation, and regression coverage around the current Flat Collection storage shape.
- The previous snapshot should stay a recovery aid, not a visible history feature or second Trash model.
- The previous snapshot boundary should cover active Notes and Recently Deleted together, while leaving Backup Health, Theme preference, and Responsive Navigation preference outside the user-note recovery payload.
- Recovery should be user-directed because automatic restore can hide data loss or silently reintroduce stale Notes.
- If serialisation or validation fails during save, neither current storage nor the previous snapshot should be changed.
- Previous snapshot restore is distinct from Backup Merge Import: snapshot restore is an exact corruption-recovery replacement, while Backup remains the user-selected Flat Collection merge path.
- The recovery state should behave like an Empty Collection sibling: load the app shell, explain that saved Notes could not be loaded, and offer restore previous snapshot, start empty, or import Backup actions.
- Quarantined corrupt payloads are support/debug artifacts, not visible recovery history and not part of the normal Note workflow.
- Backup nudges should stay in the compact Backup Controls area; banners are reserved for active storage recovery, not routine stale or missing Backup Health.
- 1.0 should make stale Backup Health more actionable rather than changing what stale means without usage evidence.
- Diagnostics must stay outside the primary Note workflow so Product Metadata does not return to the sidebar footer or editor chrome.
- PWA/offline app work can become a 1.1 milestone after the Notes storage and Backup recovery path is hardened.
- The release gate should reflect that Nota's highest-risk regressions are both storage correctness and user-visible browser workflows.
- Future storage work must start with migration safety from the 1.0 LocalStorage keys and preserve the existing Backup import/export story.
