---
name: Noter
description: Local-first Markdown notes with calm surfaces, warm accents, and task-stable controls.
colors:
  warm-capture-yellow: "#FFB340"
  paper-gray-100: "#F5F5F7"
  paper-gray-200: "#E8E8ED"
  paper-gray-300: "#D2D2D7"
  dim-desk-bg: "#1C1C1E"
  dim-desk-sidebar: "#2C2C2E"
  dim-desk-border: "#3A3A3C"
  graphite-text: "#111827"
  muted-text: "#6B7280"
  danger-red: "#EF4444"
  success-emerald: "#10B981"
typography:
  display:
    fontFamily: "\"Source Sans 3 Variable\", ui-sans-serif, system-ui, sans-serif"
    fontSize: "1.875rem"
    fontWeight: 700
    lineHeight: 1.25
    letterSpacing: "normal"
  title:
    fontFamily: "\"Source Sans 3 Variable\", ui-sans-serif, system-ui, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 700
    lineHeight: 1.3
    letterSpacing: "normal"
  body:
    fontFamily: "\"Source Sans 3 Variable\", ui-sans-serif, system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.75
    letterSpacing: "normal"
  label:
    fontFamily: "\"Source Sans 3 Variable\", ui-sans-serif, system-ui, sans-serif"
    fontSize: "0.6875rem"
    fontWeight: 500
    lineHeight: 1rem
    letterSpacing: "normal"
  mono:
    fontFamily: "\"Source Code Pro Variable\", ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.75
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
  full: "9999px"
spacing:
  xs: "4px"
  sm: "6px"
  md: "8px"
  lg: "12px"
  xl: "16px"
  surface-x: "24px"
  surface-y: "32px"
  footer-height: "45px"
components:
  button-primary:
    backgroundColor: "{colors.warm-capture-yellow}"
    textColor: "#FDFDFC"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
  button-secondary:
    backgroundColor: "{colors.paper-gray-200}"
    textColor: "{colors.graphite-text}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "8px 20px"
  search-input:
    backgroundColor: "rgba(17, 24, 39, 0.05)"
    textColor: "{colors.graphite-text}"
    typography: "{typography.label}"
    rounded: "{rounded.lg}"
    padding: "6px 16px 6px 40px"
  tag-chip:
    backgroundColor: "rgba(17, 24, 39, 0.05)"
    textColor: "{colors.muted-text}"
    typography: "{typography.label}"
    rounded: "{rounded.full}"
    padding: "2px 8px"
  note-row-selected:
    backgroundColor: "rgba(255, 179, 64, 0.10)"
    textColor: "{colors.graphite-text}"
    typography: "{typography.label}"
    rounded: "{rounded.sm}"
    padding: "12px 16px"
  footer-control:
    backgroundColor: "#FDFDFC"
    textColor: "{colors.muted-text}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "2px 6px"
---

# Design System: Noter

## 1. Overview

**Creative North Star: "The Local Notebook"**

Noter should feel like a dependable notebook sitting on a quiet desk: immediate, familiar, private, and hard to disrupt. The system is restrained product UI, built around a scannable sidebar, a generous Writing Surface, contextual Markdown tools, compact footers, and warm accent states that appear only when they help the task.

The visual language rejects an Apple Notes clone, a developer Markdown workbench, a folder or notebook-heavy organiser, a command-palette-first productivity shell, and a cloud-sync product. It should not use hover-only actions, permanent status chrome, generic destructive confirmations, or decorative visual noise around the Note.

**Key Characteristics:**
- Restrained palette with Warm Capture Yellow reserved for action, selection, focus, and progress.
- Light surfaces read as paper neutrals; Dark Theme reads as Dim Desk Surfaces, not inverted colors.
- Compact controls and stable 45px footers keep workflow chrome predictable.
- Markdown affordances support writing without becoming the product frame.

## 2. Colors

The palette is a quiet local note system: paper-like neutrals, tuned dark desk surfaces, and one warm capture accent.

### Primary
- **Warm Capture Yellow**: The sole accent. Use for primary actions, active segmented controls, selected Note rows, focus rings, text highlights, progress notifications, and empty-state illustration marks.

### Neutral
- **Quiet Paper Neutrals**: Use for Light Theme sidebar, dividers, split Preview background, footer chrome, and low-emphasis hover states.
- **Dim Desk Surfaces**: Use for Dark Theme root, sidebar, modal panels, and borders. These are separately tuned surfaces, never a simple inversion of Light Theme.
- **Graphite Text and Muted Text**: Use for primary reading text and secondary metadata. Keep Note content higher contrast than utility labels.

### Secondary
- **Recovery Red**: Use only for delete, clear, destructive confirmation, and error notification states.
- **Saved Emerald**: Use only for success notifications.

### Named Rules
**The One Warm Signal Rule.** Warm Capture Yellow is functional, not decorative. If it does not indicate action, selection, focus, highlight, or progress, remove it.

**The Separate Theme Rule.** Light and Dark Themes must be tuned separately for surfaces, borders, selection, Search Hint readability, and selected Note recognition.

## 3. Typography

**Display Font:** Source Sans 3 Variable with native sans fallbacks.
**Body Font:** Source Sans 3 Variable with native sans fallbacks.
**Label/Mono Font:** Source Sans 3 Variable for controls, Source Code Pro Variable only for Markdown body editing and syntax examples.

**Character:** Humanist, quiet, and legible. The type should feel familiar without copying the platform, with weight and spacing doing more work than ornamental font choices.

### Hierarchy
- **Display** (700, 1.875rem, 1.25): Note Title and Preview title.
- **Headline** (600, 1.5rem, 1.3): Empty-state and modal section headings.
- **Title** (700, 1.25rem, 1.3): Sidebar title and confirmation titles.
- **Body** (400, 1rem, 1.75): Markdown reading and writing body. Keep long prose around 65 to 75ch where constrained reading is possible.
- **Label** (500, 0.6875rem, 1rem): Footer controls, backup controls, compact segmented buttons, and dense utility text.

### Named Rules
**The Source Family Rule.** Use the local Source families for product UI and Markdown editing. Do not introduce display fonts, decorative letter spacing, or fluid type.

## 4. Elevation

Noter is flat by default. Depth is conveyed mostly through tonal layering, borders, dividers, selected-state rings, and compact spacing. Shadows are reserved for overlays, transient hints, modals, menus, tag suggestions, global notifications, and split Preview inset treatment.

### Shadow Vocabulary
- **Hint Lift** (`shadow-sm`): Search Hint and action menus that appear temporarily above the Note List without changing layout.
- **Modal Lift** (`shadow-2xl`): Modal panel only.
- **Small Lift** (`shadow-sm`): Primary buttons, tag suggestion panels, notifications, and compact mobile sidebar toggle.
- **Inset Reading Plane** (`shadow-inner`): Selected Note row and Split Preview when depth clarifies the active surface.

### Named Rules
**The Flat Until Floating Rule.** Surfaces at rest do not need shadows. Add lift only when an element floats, interrupts, or temporarily overlays the task.

## 5. Components

### Buttons
- **Shape:** Gently curved controls (6px radius).
- **Primary:** Warm Capture Yellow fill with high-contrast text, compact padding, and semibold label weight.
- **Hover / Focus:** Yellow darkens on hover; focus uses a 2px Warm Capture Yellow ring with offset.
- **Secondary / Danger:** Secondary buttons use paper-gray fills. Danger buttons use Recovery Red only for destructive confirmation.
- **Icon Buttons:** Sidebar utility icons use shared theme-aware neutral foregrounds plus matching hover foreground/background states in Light and Dark Themes. Do not rely on inherited text color for standalone icons.

### Chips
- **Style:** Rounded pills (9999px radius), soft neutral fill, muted text, compact 2px by 8px padding.
- **State:** Filter chips use Warm Capture Yellow fill. Tag chips remain secondary metadata, never primary navigation.

### Cards / Containers
- **Corner Style:** Small to medium radius (6px to 8px).
- **Background:** Root and Writing Surface stay quiet; sidebar, modal chrome, Search Hint, and Split Preview use tonal layers.
- **Shadow Strategy:** Flat by default. Overlay containers may use Hint Lift or Small Lift.
- **Border:** Thin full borders and dividers, never colored side stripes.
- **Internal Padding:** Dense utility containers use 12px to 16px; Writing Surface content uses 24px to 32px.

### Inputs / Fields
- **Style:** Search uses soft neutral fill and 8px radius. Note Title and content fields are transparent to keep the Note primary.
- **Focus:** Use clear focus rings or background shifts. Do not rely on color alone.
- **Error / Disabled:** Use explicit text and semantic color only when the state affects the workflow.

### Navigation
- **Style:** Sidebar is a persistent Note List on desktop and a full-width responsive panel on compact viewports.
- **Active State:** Selected Note rows use border, warm fill, and ring together for recognition.
- **Mobile Treatment:** Compact navigation toggle is a normal top-left control, not a floating mid-page handle.

### Signature Components
- **Editor-area Footer:** Stable 45px compact footer that owns Write, Preview, Split, and Markdown syntax help.
- **Search Hint:** Temporary popup below Search, theme-aware and readable in Light and Dark without pushing the Note List down.
- **Global Notification:** Floating, compact, transient feedback for save, Backup, and import outcomes.

## 6. Do's and Don'ts

### Do:
- **Do** keep Warm Capture Yellow rare and functional.
- **Do** keep the Note Title, Note Metadata, Writing Surface, Preview, and View Mode Controls visually connected.
- **Do** use compact 45px footers for sidebar utilities and editor View Mode Controls.
- **Do** tune Light and Dark Themes separately for text, borders, selection, and overlay readability.
- **Do** use full borders, tonal backgrounds, rings, and explicit labels for important states.

### Don't:
- **Don't** make Noter feel like an Apple Notes clone.
- **Don't** make Noter feel like a developer Markdown workbench.
- **Don't** introduce a folder or notebook-heavy organiser.
- **Don't** turn Search into a command-palette-first productivity shell.
- **Don't** imply cloud sync through persistent status chrome.
- **Don't** hide Note Actions behind hover-only affordances.
- **Don't** use persistent syntax instruction blocks, generic destructive confirmations, decorative chrome around the Writing Surface, gradient text, glassmorphism, colored side-stripe borders, or identical decorative card grids.
