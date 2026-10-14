# Deep cleanup playbook

## Done predicate
`cargo fmt --check`, `cargo clippy -p nota-core -p nota-app -p nota-ffi --all-targets --locked -- -D warnings`, and `cargo test -p nota-core -p nota-app -p nota-ffi --locked` stay green. Dead unused modules and browser-target residue in active Rust are gone. Native recovery, backup import, and transition fixtures still pass. No ADR history deleted.

## Units (riskiest unknown first among subtractive work)
1. Delete `nota-core::sample_notes` (zero callers).
2. Remove unused browser-era `storage_recovery` APIs (`decide_collection_startup`, `plan_collection_save_from_json`, `CollectionStartup`, `CollectionSavePlan`, `StorageRecoveryError`) and the `wasm32`/`dead_code` allow.
3. Inventory pass on remaining wasm/browser false claims outside ADRs; fix only active false claims.
4. Whole-predicate verify; encode any recurring check as a script if needed.

## Out of scope
ADR deletion, `workspace.rs` reshape, Windows UI rewrite, feature work, changing legacy `noter.*` backup kinds.
