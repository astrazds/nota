# Contributing to Nota

Thanks for taking an interest in Nota. Small, focused changes are easiest
to review.

## Before opening a pull request

1. Open an issue for behavior changes or substantial new work so the scope can
   be agreed first.
2. Preserve Nota's local-first boundary. Changes must not add telemetry, cloud
   sync, remote note storage, or analytics.
3. Keep the product a Markdown Note App. Search and the Note List remain the
   discovery system; do not introduce folders, notebooks, or a command palette
   as primary navigation.
4. Do not commit generated AppImages, `dist/`, `target/`, browser reports, local
   agent files, or editor state.
5. Run the complete local check:

   ```sh
   cargo fmt --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   cargo check --target wasm32-unknown-unknown --all-features
   python3 build-aux/test_package_appimage.py
   ```

   Native GTK work also needs GTK 4.22 and, for Preview/Split, WebKitGTK 6.
   Browser visual and workflow contracts additionally need `npm ci`,
   `npx playwright install chromium`, and `npm run test:browser`.

## Pull requests

Explain the problem, the chosen approach, and how you verified it. Add or
update a behavior-focused test for domain changes. By contributing, you agree
that your contribution is licensed under the repository's MIT license.
