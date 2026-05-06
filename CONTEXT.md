# Noter

Noter is a local-first Markdown note app focused on writing, finding, and organising personal notes. Markdown powers note content, but the primary product experience is a note app rather than a Markdown workbench.

## Language

**Note**:
A personal Markdown document that can be written, found, organised, pinned, previewed, and deleted.
_Avoid_: Document, file

**Note Title**:
A distinct name for a Note used for recognition, selection, and editing context.
_Avoid_: Derived heading

**Markdown Note App**:
A note-taking product where Markdown support is part of the writing experience, not the dominant product frame.
_Avoid_: Markdown workbench, developer editor

**Local-First Note Identity**:
The product's quiet visual identity: familiar note-app structure, warm accents, and calm surfaces without cloning another app.
_Avoid_: Apple Notes clone

**Preview**:
A rendered view of a Note's Markdown content.
_Avoid_: Output pane

**Writing Surface**:
The primary single-pane view where a Note is edited.
_Avoid_: Source pane

**View Mode**:
A user-selected way to see a Note, such as writing, previewing, or desktop-only split view.
_Avoid_: Layout toggle

**Tag**:
A lightweight label on a Note used for secondary organisation and filtering.
_Avoid_: Folder, primary navigation item

**Empty Collection**:
The state where the user has no Notes yet and needs a clear path to create the first one.
_Avoid_: No selection

**Flat Collection**:
The organisation model where Notes are kept in one collection and discovered through Search, the Note List, and lightweight Tags.
_Avoid_: Folders, notebooks

**Note Actions**:
The explicit controls for secondary Note operations such as pinning or deleting.
_Avoid_: Hover-only row controls

**Delete Confirmation**:
A note-specific confirmation that names the Note before it is permanently deleted.
_Avoid_: Generic destructive modal

**Note Metadata**:
Lightweight descriptive information about a Note, such as its Tags.
_Avoid_: Permanent bottom bar

**Search Hint**:
A temporary helper shown while Search is focused to explain scoped Search syntax without permanently adding sidebar noise.
_Avoid_: Permanent syntax block

**Save Status**:
A small editing-state indicator that tells the user whether Note changes are saved.
_Avoid_: Sidebar footer metadata

**Formatting Tools**:
Contextual controls that help users insert or wrap Markdown while writing.
_Avoid_: Primary toolbar

**Product Metadata**:
Version or build information that helps with support or diagnostics but is not part of the main note workflow.
_Avoid_: Sidebar footer content

**Search**:
The primary discovery control for finding Notes by title, content, or Tags.
_Avoid_: Command palette

**Note List**:
A scannable list of Notes optimised for recognition, selection, and lightweight filtering.
_Avoid_: Card grid, tag dashboard

**Theme**:
A tuned visual treatment for the app's surfaces, text, borders, selection states, and accent colours.
_Avoid_: Inverted colours

**Responsive Navigation**:
The sidebar behaviour that adapts note navigation for smaller screens.
_Avoid_: Floating expand handle

**Backup**:
A versioned local export of the Flat Collection that preserves Notes and can be stored outside the browser.
_Avoid_: Sync, cloud backup

**Backup Controls**:
Compact Export and Import actions for Backup, placed in the sidebar footer as a secondary utility row.
_Avoid_: Backup dropdown

**Merge Import**:
The safe restore behaviour that adds Notes from a Backup and replaces same-identity Notes without destructively clearing the current Flat Collection.
_Avoid_: Replace import by default

## Relationships

- A **Note** has one **Preview** when rendered.
- A **Note** has one **Note Title**.
- A **Note** can have zero or more **Tags**.
- A **Note** exposes **Note Actions** through a stable control.
- A **Note** should be named in its **Delete Confirmation**.
- A **Note** shows **Note Metadata** near its header or details surface.
- A **Preview** may show read-only **Note Metadata** so Preview and Split view keep Tags visible.
- A **Note** has a **Save Status** while it is being edited.
- A **Markdown Note App** prioritises creating, writing, finding, and organising **Notes** over exposing Markdown tooling.
- A **Flat Collection** contains all **Notes** without folders or notebooks.
- A **Local-First Note Identity** can borrow familiar note-app structure without copying Apple Notes exactly.
- A **Writing Surface** is the default **View Mode** for a **Note**.
- **Formatting Tools** support the **Writing Surface** but should not dominate the app chrome.
- A **Preview** is a **View Mode**, not the default workspace.
- A **Tag** supports filtering but does not define the primary navigation model.
- **Search** is the primary way to discover existing **Notes**.
- A **Search Hint** can explain scoped syntax while Search is focused, but should not become permanent sidebar content.
- The **Note List** should remain dense enough to scan several **Notes** at once.
- An **Empty Collection** should lead to creating the first **Note**.
- **Product Metadata** belongs outside the primary note workflow.
- Each **Theme** needs coherent surface, text, border, selection, and accent treatment.
- **Responsive Navigation** keeps the **Note List** available without interrupting the **Writing Surface**.
- A **Backup** preserves the **Flat Collection** outside browser storage.
- **Backup Controls** belong in the sidebar footer as secondary utilities, not in primary navigation.
- A **Merge Import** restores Notes from a **Backup** without destructively replacing the current **Flat Collection**.

## Example dialogue

> **Dev:** "Should the split Markdown preview be the default workspace?"
> **Domain expert:** "No — Noter is a **Markdown Note App**, so the default should foreground the **Note** and keep Markdown tooling contextual."
>
> **Dev:** "Should preview live beside the editor all the time?"
> **Domain expert:** "No — make the **Writing Surface** the default and let preview be an explicit **View Mode**."
>
> **Dev:** "Should the sidebar lead with every available tag?"
> **Domain expert:** "No — **Tags** are lightweight filters; search and the note list remain primary."
>
> **Dev:** "When there are no notes, should we show 'No Note Selected'?"
> **Domain expert:** "No — that is an **Empty Collection**, so show a direct path to create the first **Note**."
>
> **Dev:** "Should pin and delete only appear when hovering over a note row?"
> **Domain expert:** "No — **Note Actions** need a stable control so touch and keyboard users can discover them."
>
> **Dev:** "Should the UI copy Apple Notes as closely as possible?"
> **Domain expert:** "No — use a **Local-First Note Identity** that feels familiar but belongs to Noter."
>
> **Dev:** "Should save feedback live in the sidebar footer?"
> **Domain expert:** "No — **Save Status** belongs near the active **Note** and editing context."
>
> **Dev:** "Should Markdown commands always fill the top bar?"
> **Domain expert:** "No — **Formatting Tools** should be available while writing, but secondary to the **Writing Surface**."
>
> **Dev:** "Should the app version stay visible in the sidebar footer?"
> **Domain expert:** "No — **Product Metadata** belongs in a support or about surface, not primary navigation."
>
> **Dev:** "Should search become a command palette?"
> **Domain expert:** "No — **Search** is the primary discovery control for **Notes**, but not a general command system."
>
> **Dev:** "Should tags stay in a permanent bottom bar?"
> **Domain expert:** "No — **Note Metadata** belongs near the **Note** header or details, not below the writing area."
>
> **Dev:** "Should tags disappear when previewing?"
> **Domain expert:** "No — show read-only **Note Metadata** in **Preview** and Split view so organising context stays visible."
>
> **Dev:** "Should Search syntax be permanently visible below Search?"
> **Domain expert:** "No — use a **Search Hint** while Search is focused, then give the space back to the **Note List**."
>
> **Dev:** "Should Backup use a disclosure dropdown?"
> **Domain expert:** "No — **Backup Controls** are compact secondary utilities and fit in the sidebar footer."
>
> **Dev:** "Should each note row expand to show all tags by default?"
> **Domain expert:** "No — the **Note List** should stay scannable, with **Tags** shown only when they help recognition or filtering."
>
> **Dev:** "Can dark mode just invert the light colours?"
> **Domain expert:** "No — each **Theme** needs separately tuned surfaces, borders, selection states, and accents."
>
> **Dev:** "Should collapsed sidebar use a floating mid-page expand tab?"
> **Domain expert:** "No — sidebar collapse is **Responsive Navigation**, so mobile should use normal top-bar navigation."
>
> **Dev:** "Is 'Delete Note?' enough confirmation?"
> **Domain expert:** "No — the **Delete Confirmation** should name the **Note** so the user can verify what is being deleted."
>
> **Dev:** "Should the title be inferred from the first Markdown heading?"
> **Domain expert:** "No — keep a **Note Title** as a distinct name, but present it as part of the **Note** rather than a form field."
>
> **Dev:** "Should the redesign introduce notebooks?"
> **Domain expert:** "No — keep a **Flat Collection** and improve **Search**, the **Note List**, and **Tags** first."
>
> **Dev:** "Should importing a backup replace the current collection by default?"
> **Domain expert:** "No — use **Merge Import** for the first backup flow so restore remains safe by default."

## Flagged ambiguities

- "Markdown workbench" was considered as the product shape — resolved: Noter should be framed as a **Markdown Note App**.
- "Preview toggle" sounded like a layout control — resolved: preview is a **View Mode** for a **Note**.
- "Tag navigation" implied a primary organising model — resolved: a **Tag** is secondary filtering metadata.
- "No Note Selected" was used for both no selection and no notes — resolved: no notes is an **Empty Collection** with a creation path.
- "Hover controls" hid secondary operations — resolved: **Note Actions** should be exposed through a stable control.
- "Apple Notes clone" was too restrictive as a design target — resolved: use a **Local-First Note Identity**.
- "Saved" was treated as sidebar metadata — resolved: **Save Status** belongs with the active editing context.
- "Toolbar" implied primary app chrome — resolved: **Formatting Tools** are contextual writing affordances.
- "Version" was treated as primary sidebar content — resolved: **Product Metadata** belongs outside the main note workflow.
- "Search" could imply global commands — resolved: **Search** is scoped to discovering **Notes**.
- "Tags input" was placed as a bottom bar — resolved: Tags are **Note Metadata** and belong near the Note header or details surface.
- "Tags" were visible only while writing — resolved: read-only **Note Metadata** should also appear in **Preview** and Split view.
- "Search syntax" was persistent sidebar content — resolved: use a focus-time **Search Hint**.
- "Backup dropdown" added extra interaction for a utility feature — resolved: use compact sidebar-footer **Backup Controls**.
- "Note rows" were drifting toward card-like metadata blocks — resolved: the **Note List** should be dense and scannable.
- "Dark mode" was treated as inverted light styling — resolved: dark mode is a separately tuned **Theme**.
- "Collapsed sidebar" looked like a desktop feature — resolved: sidebar behaviour is **Responsive Navigation**.
- "Delete Note?" was generic — resolved: **Delete Confirmation** should identify the target **Note**.
- "Title" could be confused with the first Markdown heading — resolved: **Note Title** is a distinct Note property.
- "Folders" and "notebooks" imply a new primary organisation model — resolved: Noter uses a **Flat Collection** for now.
- "Backup import" could imply destructive replacement — resolved: backup v1 uses **Merge Import** and leaves replace import out of scope until a clear workflow needs it.
