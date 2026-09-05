const { expect, test } = require("@playwright/test");
const fs = require("node:fs");

const previousActive = {
  id: "11111111-1111-4111-8111-111111111111",
  title: "Previous active",
  content: "Known good browser snapshot",
  created: "2026-05-01T09:00:00Z",
  last_modified: "2026-05-01T10:00:00Z",
  tags: ["safe"],
  is_pinned: false,
};

const previousDeleted = {
  id: "22222222-2222-4222-8222-222222222222",
  title: "Previous deleted",
  content: "Recoverable browser snapshot",
  created: "2026-05-02T09:00:00Z",
  last_modified: "2026-05-02T10:00:00Z",
  tags: ["trash"],
  is_pinned: false,
};

const importedNote = {
  id: "33333333-3333-4333-8333-333333333333",
  title: "Imported from backup",
  content: "Backup recovery path",
  created: "2026-05-03T09:00:00Z",
  last_modified: "2026-05-03T10:00:00Z",
  tags: ["backup"],
  is_pinned: true,
};

async function seedRecovery(page, options = {}) {
  const {
    notesJson = "{not valid json",
    recentlyDeletedJson = "[]",
    previousNotes = [previousActive],
    previousRecentlyDeleted = [previousDeleted],
  } = options;

  await page.addInitScript(
    ({ notesJson, recentlyDeletedJson, previousNotes, previousRecentlyDeleted }) => {
      window.localStorage.setItem("nota-notes", notesJson);
      window.localStorage.setItem("nota-recently-deleted-notes", recentlyDeletedJson);
      window.localStorage.setItem("nota-dark-mode", "false");
      window.localStorage.setItem("nota-sidebar-open", "true");
      window.localStorage.removeItem("nota-backup-health");
      window.localStorage.removeItem("nota-notes-corrupt-last");
      window.localStorage.removeItem("nota-recently-deleted-notes-corrupt-last");

      if (previousNotes) {
        window.localStorage.setItem("nota-notes-previous", JSON.stringify(previousNotes));
      } else {
        window.localStorage.removeItem("nota-notes-previous");
      }

      if (previousRecentlyDeleted) {
        window.localStorage.setItem(
          "nota-recently-deleted-notes-previous",
          JSON.stringify(previousRecentlyDeleted),
        );
      } else {
        window.localStorage.removeItem("nota-recently-deleted-notes-previous");
      }
    },
    { notesJson, recentlyDeletedJson, previousNotes, previousRecentlyDeleted },
  );

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

async function uploadBackup(page, testInfo, contents) {
  const path = testInfo.outputPath("recovery-backup.json");
  fs.writeFileSync(path, contents);
  await page.locator('input[type="file"]').nth(1).setInputFiles(path);
}

test("corrupt startup offers recovery paths and restores the previous snapshot", async ({ page }) => {
  await seedRecovery(page);

  await expect(page.getByRole("heading", { name: "Saved Notes could not be loaded" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Restore previous snapshot" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Start empty" })).toBeVisible();
  await expect(page.getByText("Import Backup")).toBeVisible();

  await page.getByRole("button", { name: "Restore previous snapshot" }).click();

  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Previous active");
  await expect(page.getByRole("navigation", { name: "Notes sidebar" }).getByText("Previous active")).toBeVisible();

  await page.waitForFunction(() => {
    const notes = JSON.parse(window.localStorage.getItem("nota-notes") || "[]");
    const recentlyDeleted = JSON.parse(window.localStorage.getItem("nota-recently-deleted-notes") || "[]");
    return notes[0]?.title === "Previous active" && recentlyDeleted[0]?.title === "Previous deleted";
  });
});

test("start empty quarantines corrupt payloads and diagnostics report the quarantine", async ({ page }) => {
  await seedRecovery(page, {
    recentlyDeletedJson: "{also invalid",
    previousNotes: null,
    previousRecentlyDeleted: null,
  });

  await expect(page.getByRole("button", { name: "Restore previous snapshot" })).toBeDisabled();
  await page.getByRole("button", { name: "Start empty" }).click();

  await expect(page.getByRole("heading", { name: "Create your first note" })).toBeVisible();
  await page.waitForFunction(() => {
    return (
      window.localStorage.getItem("nota-notes") === "[]" &&
      window.localStorage.getItem("nota-recently-deleted-notes") === "[]" &&
      window.localStorage.getItem("nota-notes-corrupt-last") === "{not valid json" &&
      window.localStorage.getItem("nota-recently-deleted-notes-corrupt-last") === "{also invalid"
    );
  });

  await page.getByRole("button", { name: "About Nota" }).click();
  await expect(page.getByRole("dialog", { name: "About Nota" })).toContainText("Local browser storage");
  await expect(page.getByRole("dialog", { name: "About Nota" })).toContainText("Corrupt payload quarantined");
});

test("Backup import remains available from storage recovery", async ({ page }, testInfo) => {
  await seedRecovery(page, {
    previousNotes: null,
    previousRecentlyDeleted: null,
  });

  await uploadBackup(page, testInfo, backupJson([importedNote]));
  await expect(page.getByText("Import 1 notes: 1 new, 0 replace")).toBeVisible();
  await page.locator("button").filter({ hasText: /^Import$/ }).click();

  await expect(page.getByRole("status")).toContainText("Backup imported");
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Imported from backup");
  await page.waitForFunction(() => {
    const notes = JSON.parse(window.localStorage.getItem("nota-notes") || "[]");
    return notes.length === 1 && notes[0].title === "Imported from backup";
  });
});
