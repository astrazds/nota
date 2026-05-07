const { expect, test } = require("@playwright/test");

const baseNote = {
  id: "11111111-1111-4111-8111-111111111111",
  title: "Architecture note",
  content: "# Architecture note\n\nThe Writing Surface keeps the Note central.",
  created: "2026-05-06T09:00:00Z",
  last_modified: "2026-05-06T10:00:00Z",
  tags: ["product", "writing"],
  is_pinned: false,
};

async function seedCollection(page, { notes = [baseNote], recentlyDeleted = [] } = {}) {
  await page.addInitScript(
    ({ notes, recentlyDeleted }) => {
      window.localStorage.setItem("noter-notes", JSON.stringify(notes));
      window.localStorage.setItem("noter-recently-deleted-notes", JSON.stringify(recentlyDeleted));
      window.localStorage.setItem("noter-dark-mode", "false");
      window.localStorage.setItem("noter-sidebar-open", "true");
      window.localStorage.removeItem("noter-backup-health");
    },
    { notes, recentlyDeleted },
  );
  await page.goto("/");
}

async function waitForSaved(page) {
  await expect(page.getByRole("status")).toContainText(/Saving|Saved/);
  await expect(page.getByRole("status")).toBeHidden({ timeout: 5_000 });
}

async function waitForStatus(page, message) {
  await expect(page.getByRole("status")).toContainText(message);
  await expect(page.getByRole("status")).toBeHidden({ timeout: 5_000 });
}

function noteRow(page, title) {
  return page
    .getByRole("navigation", { name: "Notes sidebar" })
    .getByText(title, { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'group')][1]");
}

test("user can create, edit, search, and save a Note", async ({ page }) => {
  await seedCollection(page, { notes: [] });

  await page.getByRole("button", { name: "New Note", exact: true }).click();

  await expect(page.getByPlaceholder("Note Title")).toBeFocused();
  await page.getByPlaceholder("Note Title").fill("Launch checklist");
  await page.getByPlaceholder("Start typing...").fill("Ship backup coverage and note workflow tests.");
  await page.getByPlaceholder("Add tags").fill("release, QA");
  await page.getByPlaceholder("Start typing...").click();
  await waitForSaved(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await expect(navigation.getByText("Launch checklist", { exact: true })).toBeVisible();
  await expect(navigation.getByText("#release")).toBeVisible();
  await expect(navigation.getByText("#QA")).toBeVisible();

  await page.getByPlaceholder("Search").fill("tag:qa backup");
  await expect(navigation.getByText("Launch checklist", { exact: true })).toBeVisible();

  const savedNotes = await page.evaluate(() => JSON.parse(window.localStorage.getItem("noter-notes")));
  expect(savedNotes).toHaveLength(1);
  expect(savedNotes[0]).toMatchObject({
    title: "Launch checklist",
    content: "Ship backup coverage and note workflow tests.",
    tags: ["release", "QA"],
  });
});

test("user can edit an existing Note and use Preview without losing metadata", async ({ page }) => {
  await seedCollection(page);

  await page.getByPlaceholder("Note Title").fill("Architecture decision");
  await page.getByPlaceholder("Start typing...").fill("# Architecture decision\n\nPreview keeps tags visible.");
  await page.getByRole("button", { name: "Edit tags" }).click();
  await page.getByPlaceholder("Add tags").fill("product, preview");
  await page.getByPlaceholder("Start typing...").click();
  await waitForSaved(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await expect(navigation.getByText("Architecture decision", { exact: true })).toBeVisible();

  await page.getByRole("button", { name: "Preview mode" }).click();
  const preview = page.locator(".prose").first();
  await expect(preview.getByRole("heading", { name: "Architecture decision" })).toBeVisible();
  await expect(preview.getByText("#product")).toBeVisible();
  await expect(preview.getByText("#preview")).toBeVisible();
  await expect(preview.getByText("Preview keeps tags visible.")).toBeVisible();
});

test("user can delete, restore, and permanently clear Recently Deleted Notes", async ({ page }) => {
  await seedCollection(page);

  await noteRow(page, "Architecture note").getByRole("button", { name: "Note actions" }).click();
  await page.getByRole("button", { name: "Delete" }).click();

  let dialog = page.getByRole("dialog");
  await expect(dialog).toHaveAttribute("aria-labelledby", "confirmation-modal-title");
  await expect(dialog).toHaveAttribute("aria-describedby", "confirmation-modal-message");
  await expect(dialog.getByRole("heading", { name: "Move to Recently Deleted?" })).toBeVisible();
  await expect(dialog).toContainText('"Architecture note" will move to Recently Deleted.');
  await expect(dialog.getByRole("button", { name: "Cancel" })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(noteRow(page, "Architecture note").getByRole("button", { name: "Note actions" })).toBeFocused();

  await noteRow(page, "Architecture note").getByRole("button", { name: "Note actions" }).click();
  await page.getByRole("button", { name: "Delete" }).click();
  dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "Delete" }).click();
  await waitForSaved(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await expect(navigation.getByText("Recently Deleted (1)")).toBeVisible();
  await navigation.getByText("Recently Deleted (1)").click();
  await expect(navigation.getByText("Architecture note", { exact: true })).toBeVisible();

  await navigation.getByRole("button", { name: "Restore" }).click();
  await waitForSaved(page);
  await expect(noteRow(page, "Architecture note")).toBeVisible();

  await noteRow(page, "Architecture note").getByRole("button", { name: "Note actions" }).click();
  await page.getByRole("button", { name: "Delete" }).click();
  dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "Delete" }).click();
  await waitForSaved(page);

  await navigation.getByText("Recently Deleted (1)").click();
  await navigation.getByRole("button", { name: "Clear All" }).click();
  dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading", { name: "Permanently clear Recently Deleted?" })).toBeVisible();
  await expect(dialog).toContainText("This will permanently clear 1 recently deleted Note.");
  await expect(dialog.getByRole("button", { name: "Cancel" })).toBeFocused();
  await dialog.getByRole("button", { name: "Cancel" }).click();
  await expect(navigation.getByRole("button", { name: "Clear All" })).toBeFocused();
  await expect(navigation.getByText("Recently Deleted (1)")).toBeVisible();

  await navigation.getByRole("button", { name: "Clear All" }).click();
  dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "Clear All" }).click();
  await waitForStatus(page, "Recently Deleted cleared");
  await expect(navigation.getByText("Recently Deleted")).toBeHidden();
});
