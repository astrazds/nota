const { expect, test } = require("@playwright/test");

const firstNote = {
  id: "11111111-1111-4111-8111-111111111111",
  title: "Mobile capture",
  content: "Compact Note List behaviour",
  created: "2026-05-01T09:00:00Z",
  last_modified: "2026-05-01T10:00:00Z",
  tags: ["mobile"],
  is_pinned: false,
};

const secondNote = {
  id: "22222222-2222-4222-8222-222222222222",
  title: "Second note",
  content: "Selecting this note should reveal the Writing Surface.",
  created: "2026-05-02T09:00:00Z",
  last_modified: "2026-05-02T10:00:00Z",
  tags: [],
  is_pinned: false,
};

async function seedCollection(page, { notes = [firstNote, secondNote], sidebarOpen = true } = {}) {
  await page.addInitScript(
    ({ notes, sidebarOpen }) => {
      window.localStorage.setItem("nota-notes", JSON.stringify(notes));
      window.localStorage.setItem("nota-recently-deleted-notes", "[]");
      window.localStorage.setItem("nota-dark-mode", "false");
      window.localStorage.setItem("nota-sidebar-open", JSON.stringify(sidebarOpen));
      window.localStorage.removeItem("nota-backup-health");
    },
    { notes, sidebarOpen },
  );
  await page.goto("/");
}

async function expectNoteListOpen(noteList) {
  await expect(noteList).toHaveClass(/translate-x-0/);
  await expect(noteList).toHaveClass(/w-full/);
}

async function expectNoteListClosed(noteList) {
  await expect(noteList).toHaveClass(/-translate-x-full/);
  await expect(noteList).toHaveClass(/w-0/);
}

test("compact navigation opens, selects a Note, and returns to the Writing Surface", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedCollection(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await expectNoteListOpen(navigation);
  await expect(navigation.getByText("Second note", { exact: true })).toBeVisible();

  await navigation.getByText("Second note", { exact: true }).click();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Second note");
  await expectNoteListClosed(navigation);

  await page.getByRole("button", { name: "Toggle sidebar" }).click();
  await expectNoteListOpen(navigation);
  await expect(navigation.getByText("Second note", { exact: true })).toBeVisible();
});

test("compact Quick Capture hides the Note List and focuses an empty Note Title", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedCollection(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await page.getByRole("button", { name: "Create new note" }).click();

  await expectNoteListClosed(navigation);
  await expect(page.getByPlaceholder("Note Title")).toBeFocused();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("");
});

test("compact view normalises Split mode back to Write mode", async ({ page }) => {
  await seedCollection(page);

  await page.getByRole("button", { name: "Split mode" }).click();
  await expect(page.getByPlaceholder("Start typing...")).toBeVisible();
  await expect(page.locator(".prose").first()).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });

  await expect(page.getByRole("button", { name: "Split mode" })).toBeHidden();
  await expect(page.getByRole("button", { name: "Write mode" })).toHaveAttribute("aria-pressed", "");
  await expect(page.locator(".prose").first()).toBeHidden();
  await expect(page.getByPlaceholder("Start typing...")).toBeVisible();
});

test("compact sidebar preference survives a reload", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedCollection(page, { sidebarOpen: false });

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await expectNoteListClosed(navigation);

  await page.getByRole("button", { name: "Toggle sidebar" }).click();
  await expectNoteListOpen(navigation);

  const stored = await page.evaluate(() => JSON.parse(window.localStorage.getItem("nota-sidebar-open")));
  expect(stored).toBe(true);
});
