const { expect, test } = require("@playwright/test");
const fs = require("node:fs");

const existingNote = {
  id: "11111111-1111-4111-8111-111111111111",
  title: "Existing note",
  content: "Current browser copy",
  created: "2026-05-01T09:00:00Z",
  last_modified: "2026-05-01T10:00:00Z",
  tags: ["local"],
  is_pinned: false,
};

const importedReplacement = {
  ...existingNote,
  title: "Imported replacement",
  content: "Backup copy wins for same identity",
  tags: ["backup"],
  last_modified: "2026-05-02T10:00:00Z",
};

const importedOnly = {
  id: "22222222-2222-4222-8222-222222222222",
  title: "Imported only",
  content: "New backup note",
  created: "2026-05-03T09:00:00Z",
  last_modified: "2026-05-03T10:00:00Z",
  tags: ["restore"],
  is_pinned: true,
};

async function seedCollection(page, notes = [existingNote]) {
  await page.addInitScript((notes) => {
    window.localStorage.setItem("noter-notes", JSON.stringify(notes));
    window.localStorage.setItem("noter-recently-deleted-notes", "[]");
    window.localStorage.setItem("noter-dark-mode", "false");
    window.localStorage.setItem("noter-sidebar-open", "true");
    window.localStorage.removeItem("noter-backup-health");
  }, notes);
  await page.goto("/");
}

function backupJson(notes) {
  return JSON.stringify(
    {
      version: 1,
      kind: "noter.flat_collection",
      notes,
    },
    null,
    2,
  );
}

async function uploadBackup(page, testInfo, contents, name = "backup.json") {
  const path = testInfo.outputPath(name);
  fs.writeFileSync(path, contents);
  await page.locator('input[type="file"]').setInputFiles(path);
}

async function waitForSavedNoteCount(page, count) {
  await page.waitForFunction(
    (count) => JSON.parse(window.localStorage.getItem("noter-notes") || "[]").length === count,
    count,
  );
}

test("user can export a Backup and see Backup Health update", async ({ page }) => {
  await seedCollection(page);

  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export" }).click();
  const download = await downloadPromise;

  expect(download.suggestedFilename()).toMatch(/^noter-backup-\d{4}-\d{2}-\d{2}\.json$/);
  await expect(page.getByRole("status")).toContainText("Backup exported");
  await expect(page.getByText("Up to date")).toBeVisible();
});

test("final web release exports a desktop transition bundle", async ({ page }) => {
  await seedCollection(page);

  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export for desktop" }).click();
  const download = await downloadPromise;

  expect(download.suggestedFilename()).toMatch(
    /^noter-desktop-transition-\d{4}-\d{2}-\d{2}\.json$/,
  );
  const downloadPath = await download.path();
  const transition = JSON.parse(fs.readFileSync(downloadPath, "utf8"));
  expect(transition.kind).toBe("noter.desktop_transition");
  expect(transition.version).toBe(1);
  expect(transition.notes).toEqual([existingNote]);
  expect(transition.recently_deleted_notes).toEqual([]);
  expect(transition.theme).toBe("light");
  expect(transition.backup_health).toBeNull();
  await expect(page.getByRole("status")).toContainText("Desktop transition exported");
});

test("user previews, cancels, and confirms a Merge Import", async ({ page }, testInfo) => {
  await seedCollection(page);

  await uploadBackup(page, testInfo, backupJson([importedReplacement, importedOnly]));
  await expect(page.getByText("Import 2 notes: 1 new, 1 replace")).toBeVisible();

  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(page.getByText("Import 2 notes: 1 new, 1 replace")).toBeHidden();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Existing note");

  await uploadBackup(page, testInfo, backupJson([importedReplacement, importedOnly]), "backup-again.json");
  await expect(page.getByText("Import 2 notes: 1 new, 1 replace")).toBeVisible();
  await page.locator("button").filter({ hasText: /^Import$/ }).click();

  await expect(page.getByRole("status")).toContainText("Backup imported");
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Imported replacement");
  await expect(page.getByRole("navigation", { name: "Notes sidebar" }).getByText("Imported only")).toBeVisible();
  await waitForSavedNoteCount(page, 2);

  const savedNotes = await page.evaluate(() => JSON.parse(window.localStorage.getItem("noter-notes")));
  expect(savedNotes).toHaveLength(2);
  expect(savedNotes.map((note) => note.title)).toEqual(["Imported replacement", "Imported only"]);
});

test("invalid Backup import fails without changing the Flat Collection", async ({ page }, testInfo) => {
  await seedCollection(page);

  await uploadBackup(page, testInfo, "{not valid json", "invalid-backup.json");

  await expect(page.getByRole("status")).toContainText("Backup import failed");
  await expect(page.getByText("Import 2 notes")).toBeHidden();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Existing note");

  const savedNotes = await page.evaluate(() => JSON.parse(window.localStorage.getItem("noter-notes")));
  expect(savedNotes).toHaveLength(1);
  expect(savedNotes[0].title).toBe("Existing note");
});
