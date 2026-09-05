# Product

## Register

product

## Users

Nota is for people who want a fast, private, local-first place to capture, write, find, organise, preview, delete, recover, and back up personal Markdown notes. They are usually in a writing or retrieval workflow: adding a thought quickly, editing an existing Note, searching by title/content/Tags, checking rendered Markdown, or protecting their local collection with a Backup.

Users should not need to think in terms of files, folders, cloud sync, command systems, or Markdown tooling first. The product should feel like a focused note app that happens to support Markdown well.

## Product Purpose

Nota exists to make local note-taking feel dependable and low-friction on the user's own machine. Success means a user can create a Note quickly, stay oriented in a Flat Collection, edit the Writing Surface without chrome getting in the way, preview Markdown when needed, recover accidental deletes, and export/import a local Backup without risking the current collection.

The post-1.0 product surface is the Linux Relm4/GTK4 window. The 1.0.2 browser app remains a migration Adapter until native cutover. The primary product frame is a Markdown Note App, not a Markdown workbench. Markdown powers the content, but the main product experience is creating, recognising, finding, organising, and safely preserving Notes.

## Brand Personality

Calm, local-first, practical.

The interface should feel quiet, familiar, and trustworthy: a note app with its own Local-First Note Identity, warm accents, readable surfaces, stable controls, and direct recovery paths. It should project expert restraint rather than novelty. Copy should be plain and operational, using the product language from `CONTEXT.md`.

## Anti-references

Do not make Nota feel like an Apple Notes clone, a developer Markdown workbench, a folder or notebook-heavy organiser, a command-palette-first productivity shell, or a cloud-sync product.

Avoid hover-only actions, persistent syntax instruction blocks, permanent utility/status chrome, destructive import defaults, generic destructive confirmations, visual clutter around the Writing Surface, and UI patterns that make Tags feel like primary navigation.

## Design Principles

1. Keep the Note primary. The Note Title, Note Metadata, Writing Surface, Preview, and View Mode Controls should always feel like parts of the same Note workflow.
2. Optimise for fast capture and recovery. Creating a Note, confirming deletion, restoring Recently Deleted Notes, and exporting/importing Backups must be direct, explicit, and hard to misread.
3. Make discovery scannable. Search and the Note List are the primary discovery system, with Tags as lightweight filters rather than a competing hierarchy.
4. Keep Markdown contextual. Formatting Tools, Preview, Split, and Markdown help should support writing without turning the product into an editor showcase.
5. Preserve quiet local confidence. Use familiar note-app structure, warm accents, readable Light/Dark Themes, stable controls, and transient feedback instead of decorative chrome.

## Accessibility & Inclusion

Target WCAG AA for contrast and interaction states. Interactive controls should be discoverable by keyboard, pointer, and touch users, with stable Note Actions rather than hover-only affordances. Light and Dark Themes should be tuned separately for text, borders, selection states, Search Hint readability, and selected Note recognition.

Support reduced-motion-safe interactions, avoid relying on color alone for destructive or selected states, preserve readable wrapping/truncation on compact viewports, and keep confirmations specific enough that users can verify the Note or collection impact before destructive actions. Native uses GTK accessible roles and names; the browser Adapter uses ARIA labels and panel-owned dialog semantics. Selecting a Note in the Note List should keep that row in view.
