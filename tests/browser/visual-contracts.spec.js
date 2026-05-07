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

async function builtCss(page) {
  return page.evaluate(async () => {
    const hrefs = Array.from(document.querySelectorAll("link[rel='stylesheet']"), (link) => link.href);
    const styles = await Promise.all(hrefs.map((href) => fetch(href).then((response) => response.text())));
    return styles.join("\n");
  });
}

test("Tailwind build emits the critical utility classes used by visual recipes", async ({ page }) => {
  await seedNotes(page);

  const css = await builtCss(page);

  expect(css).toContain(".h-\\[45px\\]");
  expect(css).toContain(".min-w-\\[2\\.25rem\\]");
  expect(css).toContain(".bg-apple-yellow\\/10");
  expect(css).toContain(".dark\\:bg-apple-yellow\\/20");
  expect(css).toContain("Source Sans 3 Variable");
  expect(css).toContain("Source Code Pro Variable");
  expect(css).toContain("fonts/source-sans-3/source-sans-3-latin-wght-normal.woff2");
  expect(css).toContain("fonts/source-code-pro/source-code-pro-latin-wght-normal.woff2");
  expect(css).not.toContain(".noter-footer-height");
  expect(css).not.toContain(".min-w-9");
  expect(css).not.toContain(".bg-apple-yellow\\/15");
});

test("typography uses local Source families for UI and Markdown editing", async ({ page }) => {
  await seedNotes(page);

  const families = await page.evaluate(() => {
    const title = document.querySelector('textarea[placeholder="Note Title"]');
    const editor = document.querySelector('textarea[placeholder="Start typing..."]');
    return {
      body: getComputedStyle(document.body).fontFamily,
      title: getComputedStyle(title).fontFamily,
      editor: getComputedStyle(editor).fontFamily,
    };
  });

  expect(families.body).toContain("Source Sans 3 Variable");
  expect(families.title).toContain("Source Sans 3 Variable");
  expect(families.editor).toContain("Source Code Pro Variable");
});

test("desktop editor tag chips stay compact while keeping remove controls visible", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await seedNotes(page);

  const removeProduct = page.getByRole("button", { name: "Remove tag product" });
  const removeBox = await removeProduct.evaluate((element) => element.getBoundingClientRect().toJSON());
  const chipBox = await removeProduct.evaluate((element) =>
    element.parentElement.getBoundingClientRect().toJSON(),
  );

  expect(removeBox.height).toBeLessThanOrEqual(24);
  expect(removeBox.width).toBeLessThanOrEqual(24);
  expect(chipBox.height).toBeLessThanOrEqual(28);
});

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
    .locator("xpath=ancestor::div[contains(@class, 'h-[45px]')][1]");
  const backupFooter = page
    .getByRole("button", { name: "Export" })
    .locator("xpath=ancestor::div[contains(@class, 'h-[45px]')][1]");

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

test("mobile editor chrome avoids title collisions and keeps touch controls reachable", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedNotes(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await navigation.getByText("Architecture note", { exact: true }).click();

  const sidebarToggle = page.getByRole("button", { name: "Toggle sidebar" });
  const title = page.getByPlaceholder("Note Title");
  const titleAndToggle = await Promise.all([
    title.evaluate((element) => element.getBoundingClientRect().toJSON()),
    sidebarToggle.evaluate((element) => element.getBoundingClientRect().toJSON()),
  ]);
  expect(titleAndToggle[0].left).toBeGreaterThanOrEqual(titleAndToggle[1].right + 8);

  const mobileTouchTargets = [
    sidebarToggle,
    page.getByRole("button", { name: "Remove tag product" }),
    page.getByRole("button", { name: "Remove tag writing" }),
    page.getByRole("button", { name: "Write mode" }),
    page.getByRole("button", { name: "Preview mode" }),
    page.getByRole("button", { name: "Show markdown cheatsheet" }),
  ];

  for (const target of mobileTouchTargets) {
    const box = await target.evaluate((element) => element.getBoundingClientRect().toJSON());
    expect(Math.min(box.width, box.height)).toBeGreaterThanOrEqual(36);
  }

  const body = page.getByPlaceholder("Start typing...");
  await body.fill(`${note.content}\n\nMobile notification placement.`);
  const notification = page.getByRole("status");
  await expect(notification).toContainText(/Saving|Saved/);

  const notificationBox = await notification.evaluate((element) =>
    element.getBoundingClientRect().toJSON(),
  );
  const toggleBox = await sidebarToggle.evaluate((element) => element.getBoundingClientRect().toJSON());
  expect(notificationBox.top).toBeGreaterThanOrEqual(toggleBox.bottom + 8);
});

test("desktop Writing Surface keeps a readable editing measure inside the wide workspace", async ({ page }) => {
  await seedNotes(page);

  const body = page.getByPlaceholder("Start typing...");
  const measure = await body.evaluate((element) => {
    const styles = getComputedStyle(element);
    return {
      width: element.getBoundingClientRect().width,
      maxWidth: styles.maxWidth,
    };
  });

  expect(measure.width).toBeLessThanOrEqual(920);
  expect(measure.maxWidth).not.toBe("none");
});
