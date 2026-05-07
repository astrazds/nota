const { expect, test } = require("@playwright/test");

const note = {
  id: "11111111-1111-4111-8111-111111111111",
  title: "Architecture note",
  content: "# Architecture note\n\nThe Writing Surface keeps the Note central.",
  created: "2026-05-06T09:00:00Z",
  last_modified: "2026-05-06T10:00:00Z",
  tags: ["product", "writing"],
  is_pinned: true,
};

async function seedNotes(page, { dark = false } = {}) {
  await page.addInitScript(
    ({ note, dark }) => {
      window.localStorage.setItem("noter-notes", JSON.stringify([note]));
      window.localStorage.setItem("noter-recently-deleted-notes", "[]");
      window.localStorage.setItem("noter-dark-mode", JSON.stringify(dark));
      window.localStorage.setItem("noter-sidebar-open", "true");
      window.localStorage.removeItem("noter-backup-health");
    },
    { note, dark },
  );
  await page.goto("/");
}

async function seedEmptyCollection(page) {
  await page.addInitScript(() => {
    window.localStorage.setItem("noter-notes", "[]");
    window.localStorage.setItem("noter-recently-deleted-notes", "[]");
    window.localStorage.setItem("noter-dark-mode", "false");
    window.localStorage.setItem("noter-sidebar-open", "true");
    window.localStorage.removeItem("noter-backup-health");
  });
  await page.goto("/");
}

async function visibleContrast(locator) {
  return locator.evaluate((element) => {
    const styles = getComputedStyle(element);
    return {
      color: styles.color,
      backgroundColor: styles.backgroundColor,
      borderColor: styles.borderColor,
    };
  });
}

test("Light and Dark Theme keep core visual contracts readable", async ({ page }) => {
  await seedNotes(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await expect(navigation.getByText("Architecture note", { exact: true })).toBeVisible();
  await expect(page.getByPlaceholder("Search")).toBeVisible();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Architecture note");

  await page.getByPlaceholder("Search").focus();
  const searchHint = page.getByText("Search syntax").locator("..");
  await expect(searchHint).toBeVisible();
  const lightHint = await visibleContrast(searchHint);
  expect(lightHint.color).toBe("rgb(17, 24, 39)");
  expect(lightHint.backgroundColor).toBe("rgb(255, 255, 255)");

  const selectedRow = navigation
    .getByText("Architecture note", { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'group')][1]");
  const selectedStyle = await visibleContrast(selectedRow);
  expect(selectedStyle.borderColor).not.toBe("rgba(0, 0, 0, 0)");
  expect(selectedStyle.borderColor).not.toBe("rgb(229, 231, 235)");

  await page.getByRole("button", { name: "Switch to dark mode" }).click();
  await page.getByPlaceholder("Search").focus();
  const darkHint = await visibleContrast(searchHint);
  expect(darkHint.color).toBe("rgb(255, 255, 255)");
  expect(darkHint.backgroundColor).not.toBe("rgba(0, 0, 0, 0)");
});

test("Quick Capture keeps a new Note title editable instead of resetting to display fallback", async ({ page }) => {
  await seedEmptyCollection(page);

  await page.getByRole("button", { name: "New Note", exact: true }).click();

  const title = page.getByPlaceholder("Note Title");
  await expect(title).toBeFocused();
  await expect(title).toHaveValue("");

  await title.fill("Daily plan");
  await expect(title).toHaveValue("Daily plan");
  await expect(
    page.getByRole("navigation", { name: "Notes sidebar" }).getByText("Daily plan"),
  ).toBeVisible();

  await title.fill("");
  await expect(title).toHaveValue("");
  await expect(
    page.getByRole("navigation", { name: "Notes sidebar" }).getByText("New Note"),
  ).toBeVisible();
});

test("Preview, Split, footers, Backup Controls, and notifications keep their layout contracts", async ({ page }) => {
  await seedNotes(page);

  const editorFooter = page
    .getByRole("button", { name: "Write mode" })
    .locator("xpath=ancestor::div[contains(@class, 'noter-footer-height')][1]");
  const backupFooter = page
    .getByRole("button", { name: "Export" })
    .locator("xpath=ancestor::div[contains(@class, 'noter-footer-height')][1]");

  const footerHeights = await Promise.all([
    editorFooter.evaluate((element) => element.getBoundingClientRect().height),
    backupFooter.evaluate((element) => element.getBoundingClientRect().height),
  ]);
  expect(footerHeights[0]).toBe(45);
  expect(footerHeights[1]).toBe(45);

  await page.getByRole("button", { name: "Split mode" }).click();
  const preview = page.locator(".prose").first();
  await expect(preview.getByRole("heading", { name: "Architecture note" })).toBeVisible();
  await expect(preview.getByText("#product")).toBeVisible();
  await expect(preview.getByText("#writing")).toBeVisible();
  await expect(preview.getByText("The Writing Surface keeps the Note central.")).toBeVisible();

  const previewOrder = await preview.evaluate((element) => {
    const title = element.querySelector("h1").compareDocumentPosition(element.querySelector(".not-prose"));
    const tags = element.querySelector(".not-prose").compareDocumentPosition(element.querySelector("p"));
    return {
      titleBeforeTags: Boolean(title & Node.DOCUMENT_POSITION_FOLLOWING),
      tagsBeforeBody: Boolean(tags & Node.DOCUMENT_POSITION_FOLLOWING),
    };
  });
  expect(previewOrder).toEqual({ titleBeforeTags: true, tagsBeforeBody: true });

  const body = page.getByPlaceholder("Start typing...");
  await body.fill(`${note.content}\n\nSaved by browser coverage.`);
  await expect(page.getByRole("status")).toContainText(/Saving|Saved/);
  await expect(page.getByRole("status")).toBeHidden({ timeout: 5_000 });
});
