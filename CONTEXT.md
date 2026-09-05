# Noter

Noter is a local-first Markdown note app focused on writing, finding, and organising personal notes. Markdown powers note content, but the primary product experience is a note app rather than a Markdown workbench.

## Language

**Note**:
A personal Markdown document that can be quickly captured, written, found, organised, pinned, previewed, and deleted.
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
A rendered view of a Note that shows the Note Title, read-only Note Metadata, and Markdown body.
_Avoid_: Output pane

**Writing Surface**:
The primary single-pane view where a Note is edited.
_Avoid_: Source pane

**Pane Rhythm**:
The shared inset, reading measure, type scale, and footer height that make Write, Preview, and Split feel like one workspace.
_Avoid_: Per-mode layout personality

**View Mode**:
A user-selected way to see a Note, such as writing, previewing, or desktop-only split view.
_Avoid_: Layout toggle

**View Mode Controls**:
The compact controls for switching between Write, Preview, Split, and Markdown help, kept with the editor area at a stable height.
_Avoid_: Primary app header

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

**Quick Capture**:
A fast creation path that starts a new Note, selects it, and focuses the Note Title without making the user manage navigation first.
_Avoid_: New document wizard

**Delete Confirmation**:
A note-specific confirmation that names the Note before it moves to Recently Deleted.
_Avoid_: Generic destructive modal

**Recently Deleted**:
A recoverable holding area for Notes deleted from the Flat Collection, with explicit Restore and Clear actions.
_Avoid_: Hidden undo state, Trash as primary navigation

**Clear All**:
An explicit Recently Deleted action that permanently clears all recoverable Notes after confirmation.
_Avoid_: Empty, silent purge

**Note Metadata**:
Lightweight descriptive information about a Note, such as its Tags.
_Avoid_: Permanent bottom bar

**Search Hint**:
A temporary helper shown while Search is focused to explain scoped Search syntax without permanently adding sidebar noise.
_Avoid_: Permanent syntax block

**Save Status**:
A small editing-state indicator that tells the user whether Note changes are saved.
_Avoid_: Sidebar footer metadata

**Global Notification**:
A short-lived floating message for save, Backup, and import feedback that does not create persistent header chrome.
_Avoid_: Permanent footer status text, app header status

**Formatting Tools**:
Contextual controls that help users insert or wrap Markdown while writing.
_Avoid_: Primary toolbar

**Product Metadata**:
Version or build information that helps with support or diagnostics but is not part of the main note workflow.
_Avoid_: Sidebar footer content

**Search**:
The primary discovery control for finding Notes by title, content, or Tags.
_Avoid_: Command palette

**Discovery Depth**:
Improvements that help users find a Note they vaguely remember inside the Flat Collection, led by Search and supported by richer Note List feedback.
_Avoid_: Folder hierarchy, semantic search by default

**Match Snippet**:
A compact Note List preview excerpt shown around a body Search match when the Note Title or visible Tags do not already explain why a Note matched. Match Snippets use plain and quoted Search terms, not scoped title or Tag terms, and should prefer the snippet window that explains the most matched terms while keeping the Note List scannable.
_Avoid_: Full result excerpt, multiple expanded matches

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
A versioned local export of the Flat Collection that preserves Notes and can be stored outside the live collection.
_Avoid_: Sync, cloud backup

**Backup Health**:
Lightweight metadata about the last successful Backup export, used to show whether the user has a recent local recovery point.
_Avoid_: Sync status

**Storage Recovery**:
The startup state shown when saved Notes or Recently Deleted payloads cannot be parsed, requiring the user to choose Restore previous snapshot, Start empty, or Import Backup before normal editing resumes.
_Avoid_: Silent reset, automatic data loss

**Previous Snapshot**:
The last valid active Notes and Recently Deleted collection pair preserved before a safe save writes the next collection.
_Avoid_: Undo history, version history

**Corrupt Payload Quarantine**:
The preserved copy of corrupt saved payloads after the user chooses to start empty, used for diagnostics rather than normal app loading.
_Avoid_: Backup, recovery point

**Diagnostics Surface**:
A secondary support surface for Product Metadata, storage mode, Backup Health, and corrupt-payload quarantine state.
_Avoid_: Sidebar footer metadata, primary navigation

**Backup Controls**:
Compact Export and Import actions for Backup, placed in the sidebar footer as a secondary utility row.
_Avoid_: Backup dropdown

**Backup Import Preview**:
A confirmation step that validates a Backup and shows how many Notes a Merge Import will add or replace before applying it.
_Avoid_: Blind import

**Merge Import**:
The safe restore behaviour that adds Notes from a Backup and replaces same-identity Notes without destructively clearing the current Flat Collection.
_Avoid_: Replace import by default

## Relationships

- A **Note** has one **Preview** when rendered.
- A **Note** has one **Note Title**.
- A **Note** can have zero or more **Tags**.
- A **Note** exposes **Note Actions** through a stable control.
- A **Note** should be named in its **Delete Confirmation**.
- A **Note** can move to **Recently Deleted** before it is explicitly cleared.
- **Clear All** permanently removes every Note in **Recently Deleted** and should confirm the count before applying.
- **Clear All** should be visible in the **Recently Deleted** summary row when recoverable Notes exist.
- A **Note** shows **Note Metadata** near its header or details surface.
- A **Preview** shows the **Note Title**, then read-only **Note Metadata**, then the Markdown body so Preview and Split view match the Writing Surface header order.
- **Pane Rhythm** keeps the **Writing Surface**, full **Preview**, and Split panes aligned to the same content origin and readable measure.
- A **Note** has a **Save Status** while it is being edited.
- A **Global Notification** can show save, Backup, and import outcomes above the app chrome, then clear itself after a short delay.
- A **Markdown Note App** prioritises creating, writing, finding, and organising **Notes** over exposing Markdown tooling.
- **Quick Capture** creates a **Note** and returns the user to the **Writing Surface**.
- A **Flat Collection** contains all **Notes** without folders or notebooks.
- A **Local-First Note Identity** can borrow familiar note-app structure without copying Apple Notes exactly.
- A **Writing Surface** is the default **View Mode** for a **Note**.
- **View Mode Controls** belong in a consistent editor-area footer and should keep a stable height across Write, Preview, and Split.
- In Split, one editor-area footer should span the editor/preview area because both panes are one **View Mode** for the same **Note**.
- Desktop should avoid a persistent editor header when the editor-area footer can carry **View Mode Controls**.
- Compact viewports may keep a minimal **Responsive Navigation** toggle so the user can return to the **Note List**.
- **Formatting Tools** support the **Writing Surface** but should not dominate the app chrome.
- **Formatting Tools** should only appear when the **Writing Surface** is available.
- In the **Writing Surface**, **Formatting Tools** sit after the Note header metadata and before the Markdown body.
- A **Preview** is a **View Mode**, not the default workspace.
- A **Tag** supports filtering but does not define the primary navigation model.
- **Search** is the primary way to discover existing **Notes**.
- **Discovery Depth** should improve Search-led recognition inside the **Flat Collection** before adding new organisation models.
- A **Match Snippet** should explain body Search matches when the normal **Note List** preview would not show why a **Note** matched.
- A **Match Snippet** should not replace the normal **Note List** preview when the **Note Title** or visible **Tags** already explain the Search match.
- A **Match Snippet** should replace the normal preview line with one compact excerpt, clipped with ellipses when needed, with matched terms highlighted and no extra body-match label.
- Search match highlighting belongs in the **Note List** for this direction; selecting a **Note** should open the normal editor or preview without carrying highlight state into the document surface.
- Matched **Tags** should be visibly explanatory in the **Note List** row without becoming primary navigation.
- Active **Search** can show lightweight result status and empty-state explanation, but should not become a separate results page or filter dashboard.
- Recent Search memory is out of scope for **Discovery Depth** until current Search results explain themselves clearly.
- Keyboard navigation across filtered results is a later workflow slice, separate from first improving why each Search result matched.
- **Discovery Depth** should have mobile parity for snippets, matched Tags, result status, and empty states, with compact rendering allowed to preserve **Note List** scan density.
- The success bar for **Discovery Depth** is that a user can search a remembered phrase or Tag and understand every visible result without opening **Notes**.
- A **Search Hint** can explain scoped syntax while Search is focused, but should not become permanent sidebar content.
- The **Note List** should remain dense enough to scan several **Notes** at once.
- An **Empty Collection** should lead to creating the first **Note**.
- **Product Metadata** belongs outside the primary note workflow.
- Each **Theme** needs coherent surface, text, border, selection, and accent treatment.
- **Responsive Navigation** keeps the **Note List** available without interrupting the **Writing Surface**.
- A **Backup** preserves the **Flat Collection** outside the live collection.
- **Backup Health** describes the recency of the user's last successful **Backup** export without implying sync.
- Missing or stale **Backup Health** should be actionable in **Backup Controls** without becoming a warning banner.
- **Storage Recovery** should block normal editing until the user chooses an explicit recovery path.
- A **Previous Snapshot** covers active Notes and **Recently Deleted** together.
- **Corrupt Payload Quarantine** preserves broken saved payloads only after the user chooses to start empty.
- **Diagnostics Surface** owns **Product Metadata** and storage diagnostics outside the primary note workflow.
- **Storage Recovery** should keep **Backup Import Preview** and **Merge Import** available because Backup remains the explicit user-owned recovery mechanism.
- **Backup Controls** belong in the sidebar footer as secondary utilities, not in primary navigation.
- A **Backup Import Preview** should appear before a **Merge Import** changes the **Flat Collection**.
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
> **Dev:** "Should creating a note leave mobile users in the Note List?"
> **Domain expert:** "No — **Quick Capture** should create the **Note**, select it, and return to the **Writing Surface** with the **Note Title** ready."
>
> **Dev:** "Should the UI copy Apple Notes as closely as possible?"
> **Domain expert:** "No — use a **Local-First Note Identity** that feels familiar but belongs to Noter."
>
> **Dev:** "Should save feedback live in the sidebar footer?"
> **Domain expert:** "No — **Save Status** belongs near the active **Note** and editing context."
>
> **Dev:** "Should Backup status stay in the sidebar footer?"
> **Domain expert:** "No — use a **Global Notification** for transient save, Backup, and import feedback, then let it clear itself."
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
> **Domain expert:** "No — show read-only **Note Metadata** under the **Note Title** in **Preview** and Split view so organising context stays visible and matches the **Writing Surface**."
>
> **Dev:** "Should Preview be centred because it is read-only?"
> **Domain expert:** "No — keep the same **Pane Rhythm** as the **Writing Surface** so changing **View Mode** does not feel like moving to a separate document."
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
> **Domain expert:** "No — the **Delete Confirmation** should name the **Note** so the user can verify what is moving to **Recently Deleted**."
>
> **Dev:** "Should the title be inferred from the first Markdown heading?"
> **Domain expert:** "No — keep a **Note Title** as a distinct name, but present it as part of the **Note** rather than a form field."
>
> **Dev:** "Should the redesign introduce notebooks?"
> **Domain expert:** "No — keep a **Flat Collection** and improve **Search**, the **Note List**, and **Tags** first."
>
> **Dev:** "Should importing a backup replace the current collection by default?"
> **Domain expert:** "No — use **Merge Import** for the first backup flow so restore remains safe by default."
>
> **Dev:** "Should Backup import apply as soon as a file is selected?"
> **Domain expert:** "No — show a **Backup Import Preview** first so the user can confirm the add/replace impact."
>
> **Dev:** "Should Backup Health behave like sync status?"
> **Domain expert:** "No — **Backup Health** only reflects the last successful local export recovery point."
>
> **Dev:** "Should corrupt saved Notes fall back to starter notes or an empty collection?"
> **Domain expert:** "No — show **Storage Recovery** so the user explicitly restores a **Previous Snapshot**, starts empty, or imports a **Backup**."
>
> **Dev:** "Should starting empty discard corrupt saved payloads immediately?"
> **Domain expert:** "No — use **Corrupt Payload Quarantine** so diagnostics can still explain that recovery happened."
>
> **Dev:** "Should app version and storage mode live in the Backup footer?"
> **Domain expert:** "No — put **Product Metadata** and storage details in a **Diagnostics Surface**."

## Flagged ambiguities

- "Markdown workbench" was considered as the product shape — resolved: Noter should be framed as a **Markdown Note App**.
- "Preview toggle" sounded like a layout control — resolved: preview is a **View Mode** for a **Note**.
- "Tag navigation" implied a primary organising model — resolved: a **Tag** is secondary filtering metadata.
- "No Note Selected" was used for both no selection and no notes — resolved: no notes is an **Empty Collection** with a creation path.
- "Hover controls" hid secondary operations — resolved: **Note Actions** should be exposed through a stable control.
- "New note" behaved like a generic creation action — resolved: **Quick Capture** should start a selected Note and focus the Note Title.
- "Apple Notes clone" was too restrictive as a design target — resolved: use a **Local-First Note Identity**.
- "Saved" was treated as sidebar metadata — resolved: **Save Status** belongs with the active editing context.
- "Backup exported" was treated as persistent footer metadata — resolved: transient Backup and import feedback belongs in a floating **Global Notification**.
- "Toolbar" implied primary app chrome — resolved: **Formatting Tools** are contextual writing affordances.
- "Version" was treated as primary sidebar content — resolved: **Product Metadata** belongs outside the main note workflow.
- "Search" could imply global commands — resolved: **Search** is scoped to discovering **Notes**.
- "Tags input" was placed as a bottom bar — resolved: Tags are **Note Metadata** and belong near the Note header or details surface.
- "Tags" were visible only while writing — resolved: read-only **Note Metadata** should also appear under the **Note Title** in **Preview** and Split view.
- "Search syntax" was persistent sidebar content — resolved: use a focus-time **Search Hint**.
- "Backup dropdown" added extra interaction for a utility feature — resolved: use compact sidebar-footer **Backup Controls**.
- "Note rows" were drifting toward card-like metadata blocks — resolved: the **Note List** should be dense and scannable.
- "Dark mode" was treated as inverted light styling — resolved: dark mode is a separately tuned **Theme**.
- "Collapsed sidebar" looked like a desktop feature — resolved: sidebar behaviour is **Responsive Navigation**.
- "Delete Note?" was generic — resolved: **Delete Confirmation** should identify the target **Note** and clarify that it moves to **Recently Deleted**.
- "Deleted" implied immediate permanent loss — resolved: deleted Notes move to **Recently Deleted** until restored or explicitly cleared.
- "Title" could be confused with the first Markdown heading — resolved: **Note Title** is a distinct Note property.
- "Folders" and "notebooks" imply a new primary organisation model — resolved: Noter uses a **Flat Collection** for now.
- "Backup import" could imply destructive replacement — resolved: backup v1 uses **Merge Import** and leaves replace import out of scope until a clear workflow needs it.
- "Backup import" could feel blind — resolved: use **Backup Import Preview** to confirm add/replace impact before applying a **Merge Import**.
- "Backup status" could imply cloud sync — resolved: **Backup Health** only tracks last successful local export metadata.
- "Corrupt startup" could silently reset user data — resolved: use **Storage Recovery** with explicit Restore previous snapshot, Start empty, and Import Backup paths.
- "Previous notes" could imply full history — resolved: a **Previous Snapshot** is one last-known-good active/Recently Deleted collection pair, not version history.
- "Start empty" could destroy evidence of the corrupt payload — resolved: use **Corrupt Payload Quarantine** for diagnostics after the user chooses that path.
- "Diagnostics" could become persistent app chrome — resolved: keep Product Metadata and storage diagnostics in a secondary **Diagnostics Surface**.
