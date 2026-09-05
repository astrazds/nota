const { expect, test } = require("@playwright/test");

const launchNote = {
  id: "11111111-1111-4111-8111-111111111111",
  title: "Launch Plan",
  content: "Capture migration risks before launch.",
  created: "2026-05-01T09:00:00Z",
  last_modified: "2026-05-01T10:00:00Z",
  tags: ["Work", "Product"],
  is_pinned: false,
};

const archiveNote = {
  id: "22222222-2222-4222-8222-222222222222",
  title: "Archive",
  content: "Old launch details.",
  created: "2026-05-02T09:00:00Z",
  last_modified: "2026-05-02T10:00:00Z",
  tags: ["Reference"],
  is_pinned: false,
};

async function seedCollection(page, notes = [launchNote, archiveNote]) {
  await page.addInitScript((notes) => {
    window.localStorage.setItem("nota-notes", JSON.stringify(notes));
    window.localStorage.setItem("nota-recently-deleted-notes", "[]");
    window.localStorage.setItem("nota-dark-mode", "false");
    window.localStorage.setItem("nota-sidebar-open", "true");
    window.localStorage.removeItem("nota-backup-health");
  }, notes);
  await page.goto("/");
}

function navigation(page) {
  return page.getByRole("navigation", { name: "Notes sidebar" });
}

function noteRow(page, title) {
  return navigation(page)
    .getByText(title, { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'group')][1]");
}

async function selectText(locator, start, end) {
  await locator.evaluate(
    (textarea, { start, end }) => {
      textarea.focus();
      textarea.setSelectionRange(start, end);
    },
    { start, end },
  );
}

async function waitForSavedTags(page, tags) {
  await page.waitForFunction(
    (tags) => {
      const notes = JSON.parse(window.localStorage.getItem("nota-notes") || "[]");
      return JSON.stringify(notes[0]?.tags) === JSON.stringify(tags);
    },
    tags,
  );
}

test("user can pin Notes and combine scoped Search filters", async ({ page }) => {
  await seedCollection(page);

  await noteRow(page, "Archive").getByRole("button", { name: "Note actions" }).click();
  await page.getByRole("button", { name: "Pin" }).click();

  const rowsAfterPin = await navigation(page)
    .locator("div.group h3")
    .evaluateAll((rows) => rows.map((row) => row.textContent.trim()));
  expect(rowsAfterPin.slice(0, 2)).toEqual(["Archive", "Launch Plan"]);

  await page.getByPlaceholder("Search").fill('title:launch tag:work "migration risks"');
  await expect(navigation(page).getByText("Launch Plan", { exact: true })).toBeVisible();
  await expect(navigation(page).getByText("Archive", { exact: true })).toBeHidden();

  await page.getByPlaceholder("Search").fill("is:pinned");
  await expect(navigation(page).getByText("Archive", { exact: true })).toBeVisible();
  await expect(navigation(page).getByText("Launch Plan", { exact: true })).toBeHidden();

  await page.getByPlaceholder("Search").fill("title:missing");
  await expect(navigation(page).getByText("No notes match search: title:missing")).toBeVisible();
});

test("Search and Tag filters show result status and filtered-empty explanation", async ({ page }) => {
  await seedCollection(page);

  const sidebar = navigation(page);

  await page.getByPlaceholder("Search").fill("launch");
  await expect(sidebar.getByText("2 matches for search: launch")).toBeVisible();

  await noteRow(page, "Archive").getByRole("button", { name: "Filter by tag Reference" }).click();
  await expect(sidebar.getByText("1 match for search: launch in #Reference")).toBeVisible();

  await page.getByPlaceholder("Search").fill("migration");
  await expect(sidebar.getByText("0 matches for search: migration in #Reference")).toBeVisible();
  await expect(sidebar.getByText("No notes match search: migration in #Reference")).toBeVisible();
  await expect(sidebar.getByText("Try a different search term or clear the Tag filter.")).toBeVisible();
});

test("selected Note stays editable when hidden by Search or Tag filters", async ({ page }) => {
  await seedCollection(page);

  await page.getByPlaceholder("Search").fill("title:archive");

  await expect(navigation(page).getByText("Archive", { exact: true })).toBeVisible();
  await expect(navigation(page).getByText("Launch Plan", { exact: true })).toBeHidden();
  await expect(page.getByText("This note is outside the current Search or Tag filter.")).toBeVisible();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Launch Plan");

  await page.getByPlaceholder("Note Title").fill("Launch Plan Edited");
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Launch Plan Edited");
});

test("selecting a Search result keeps highlights out of document surfaces", async ({ page }) => {
  await seedCollection(page);

  await page.getByPlaceholder("Search").fill("old");

  const archiveRow = noteRow(page, "Archive");
  await expect(archiveRow.locator("mark", { hasText: "Old" })).toBeVisible();
  await archiveRow.click();

  const writingSurface = page
    .getByPlaceholder("Start typing...")
    .locator("xpath=ancestor::div[contains(@class, 'flex-col')][1]");
  await expect(page.getByPlaceholder("Search")).toHaveValue("old");
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Archive");
  await expect(page.getByPlaceholder("Start typing...")).toHaveValue("Old launch details.");
  await expect(writingSurface.locator("mark")).toHaveCount(0);

  await page.getByRole("button", { name: "Preview mode" }).click();
  const preview = page.locator(".prose").first();
  await expect(preview.getByRole("heading", { name: "Archive" })).toBeVisible();
  await expect(preview.getByText("Old launch details.")).toBeVisible();
  await expect(preview.locator("mark")).toHaveCount(0);

  await page.getByRole("button", { name: "Split mode" }).click();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Archive");
  await expect(page.getByPlaceholder("Start typing...")).toHaveValue("Old launch details.");
  await expect(writingSurface.locator("mark")).toHaveCount(0);
  await expect(preview.locator("mark")).toHaveCount(0);
});

test("user can accept tag suggestions and remove Tags while editing metadata", async ({ page }) => {
  await seedCollection(page);

  await navigation(page).getByText("Archive", { exact: true }).click();
  await page.getByRole("button", { name: "Edit tags" }).click();
  await page.getByPlaceholder("Add tags").fill("pro");
  await expect(page.getByRole("button", { name: "#Product" })).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.getByPlaceholder("Add tags")).toHaveValue("Product");

  await page.getByPlaceholder("Add tags").fill("");
  await page.getByPlaceholder("Start typing...").click();
  await expect(page.getByRole("button", { name: "Edit tags" })).toBeHidden();
  await expect(page.getByPlaceholder("Add tags")).toBeVisible();
});

test("user can review and apply startup Tag cleanup", async ({ page }) => {
  await seedCollection(page, [
    {
      ...launchNote,
      tags: [" Work ", "work"],
    },
  ]);

  await expect(page.getByText("Review Tag cleanup")).toBeVisible();
  await page.getByText("Review Tag cleanup").click();
  await expect(page.getByText("Work , work -> Work")).toBeVisible();
  await page.getByRole("button", { name: "Apply cleanup" }).click();
  await expect(page.getByText("Review Tag cleanup")).toBeHidden();
  await waitForSavedTags(page, ["Work"]);

  const savedNotes = await page.evaluate(() => JSON.parse(window.localStorage.getItem("nota-notes")));
  expect(savedNotes[0].tags).toEqual(["Work"]);
});

test("Formatting Tools modify Markdown around browser selections", async ({ page }) => {
  await seedCollection(page, [{ ...launchNote, content: "A😀B\nitem\n" }]);

  const body = page.getByPlaceholder("Start typing...");
  await selectText(body, 1, 3);
  await page.getByRole("button", { name: "Bold" }).click();
  await expect(body).toHaveValue("A**😀**B\nitem\n");

  await selectText(body, 9, 13);
  await page.getByRole("button", { name: "Italic" }).click();
  await expect(body).toHaveValue("A**😀**B\n*item*\n");

  await selectText(body, 16, 16);
  await page.getByRole("button", { name: "Task list" }).click();
  await expect(body).toHaveValue("A**😀**B\n*item*\n- [ ] ");

  await page.getByRole("button", { name: "Insert table" }).click();
  await expect(body).toHaveValue(/Column 1 \| Column 2/);
});

test("Preview renders Markdown features while keeping unsafe input inert", async ({ page }) => {
  await seedCollection(page, [
    {
      ...launchNote,
      title: "Preview safety",
      content:
        "# Preview safety\n\n<script>alert(1)</script>\n\n[bad](javascript:alert(1))\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n- [x] done\n\nFootnote[^1]\n\n[^1]: Footnote text",
      tags: ["preview"],
    },
  ]);

  await page.getByRole("button", { name: "Preview mode" }).click();
  const preview = page.locator(".prose").first();

  await expect(preview.getByRole("heading", { name: "Preview safety" })).toBeVisible();
  await expect(preview.getByText("<script>alert(1)</script>")).toBeVisible();
  await expect(preview.locator("script")).toHaveCount(0);
  await expect(preview.locator("a[href='javascript:alert(1)']")).toHaveCount(0);
  await expect(preview.locator("table")).toBeVisible();
  await expect(preview.locator("input[type='checkbox']")).toBeVisible();
  await expect(preview.getByText("Footnote text")).toBeVisible();
});

test("keyboard Quick Capture and Markdown help are available from the editor", async ({ page }) => {
  await seedCollection(page);
  await expect(page.getByPlaceholder("Note Title")).toBeVisible();

  await page.keyboard.press("Control+N");
  await expect(page.getByPlaceholder("Note Title")).toBeFocused();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("");

  await page.getByRole("button", { name: "Show markdown cheatsheet" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading", { name: "Markdown syntax" })).toBeVisible();
  await expect(dialog).toHaveAttribute("aria-labelledby", "markdown-syntax-title");
  await expect(dialog.getByRole("heading", { name: "Core syntax" })).toBeVisible();
  await expect(dialog.getByRole("heading", { name: "Extended syntax" })).toBeVisible();
  await expect(dialog.getByText("Raw HTML is displayed as text for safety.")).toBeVisible();
  await expect(dialog.getByText("Click backdrop or X to close.")).toHaveCount(0);
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(page.getByRole("button", { name: "Show markdown cheatsheet" })).toBeFocused();
});
