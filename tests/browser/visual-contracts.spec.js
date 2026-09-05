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
      window.localStorage.setItem("nota-notes", JSON.stringify([note]));
      window.localStorage.setItem("nota-recently-deleted-notes", "[]");
      window.localStorage.setItem("nota-dark-mode", JSON.stringify(dark));
      window.localStorage.setItem("nota-sidebar-open", "true");
      window.localStorage.removeItem("nota-backup-health");
    },
    { note, dark },
  );
  await page.goto("/");
}

async function seedEmptyCollection(page) {
  await page.addInitScript(() => {
    window.localStorage.setItem("nota-notes", "[]");
    window.localStorage.setItem("nota-recently-deleted-notes", "[]");
    window.localStorage.setItem("nota-dark-mode", "false");
    window.localStorage.setItem("nota-sidebar-open", "true");
    window.localStorage.removeItem("nota-backup-health");
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

test("typography uses local Source families for UI and Markdown editing", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await seedNotes(page);

  const title = page.getByPlaceholder("Note Title");
  const editor = page.getByPlaceholder("Start typing...");
  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  const editorFrame = page.locator('[data-testid="editor-frame"]');
  const families = {
    body: await page.evaluate(() => getComputedStyle(document.body).fontFamily),
    title: await title.evaluate((element) => getComputedStyle(element).fontFamily),
    editor: await editor.evaluate((element) => getComputedStyle(element).fontFamily),
  };
  const sizes = {
    sidebarTitle: await navigation
      .getByRole("heading", { name: "Nota" })
      .evaluate((element) => parseFloat(getComputedStyle(element).fontSize)),
    noteTitle: await title.evaluate((element) => parseFloat(getComputedStyle(element).fontSize)),
    noteListTitle: await navigation
      .getByRole("heading", { name: "Architecture note" })
      .evaluate((element) => parseFloat(getComputedStyle(element).fontSize)),
    noteListMeta: await navigation
      .getByText("06/05/2026")
      .evaluate((element) => parseFloat(getComputedStyle(element).fontSize)),
    editorBody: await editor.evaluate((element) => parseFloat(getComputedStyle(element).fontSize)),
    editorFooter: await editorFrame
      .getByText(/^Lines: \d+$/)
      .evaluate((element) => parseFloat(getComputedStyle(element).fontSize)),
  };

  expect(families.body).toContain("Source Sans 3 Variable");
  expect(families.title).toContain("Source Sans 3 Variable");
  expect(families.editor).toContain("Source Code Pro Variable");
  expect(sizes).toEqual({
    sidebarTitle: 20,
    noteTitle: 24,
    noteListTitle: 14,
    noteListMeta: 11,
    editorBody: 14,
    editorFooter: 11,
  });

  await page.getByRole("button", { name: "About Nota" }).click();
  const aboutTypography = await page.getByRole("dialog", { name: "About Nota" }).evaluate((dialog) => {
    const heading = dialog.querySelector("h2");
    const description = dialog.querySelector("#about-modal-description");
    const body = dialog.querySelector("dl");
    const close = Array.from(dialog.querySelectorAll("button")).find(
      (button) => button.textContent.trim() === "Close",
    );

    return {
      heading: parseFloat(getComputedStyle(heading).fontSize),
      description: parseFloat(getComputedStyle(description).fontSize),
      body: parseFloat(getComputedStyle(body).fontSize),
      close: parseFloat(getComputedStyle(close).fontSize),
    };
  });
  expect(aboutTypography).toEqual({
    heading: 20,
    description: 14,
    body: 14,
    close: 12,
  });
});

test("desktop editor tag chips stay compact and defer removal to tag editing", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await seedNotes(page);

  await expect(page.getByRole("button", { name: "Remove tag product" })).toBeHidden();

  const productChip = page.getByText("#product", { exact: true }).first();
  const chipBox = await productChip.evaluate((element) =>
    element.getBoundingClientRect().toJSON(),
  );

  expect(chipBox.height).toBeLessThanOrEqual(28);

  await page.getByRole("button", { name: "Edit tags" }).click();
  await expect(page.getByPlaceholder("Add tags")).toBeVisible();
  await expect(page.getByPlaceholder("Add tags")).toHaveValue("product, writing");
});

test("startup does not show a save notification before user edits", async ({ page }) => {
  await seedNotes(page);

  await page.waitForTimeout(1_000);
  await expect(page.getByRole("status")).toHaveCount(0);
});

test("desktop primary actions use recognizable labels where space allows", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await seedNotes(page);

  await expect(page.getByRole("button", { name: "Create new note" })).toContainText("New");
  await expect(page.getByRole("button", { name: "Create new note" })).toContainText("Ctrl N");
  await expect(page.getByRole("button", { name: "Focus search notes" })).not.toContainText("/");
  await expect(page.getByRole("button", { name: "Show markdown cheatsheet" })).toContainText("Help");
});

test("mobile sidebar and Backup footer controls keep touch-safe hit areas", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedNotes(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  const targets = [
    navigation.getByRole("button", { name: "Collapse sidebar" }),
    navigation.getByRole("button", { name: "Create new note" }),
    navigation.getByRole("button", { name: "Switch to dark mode" }),
    navigation.getByRole("button", { name: "Focus search notes" }),
    navigation.getByRole("button", { name: "About Nota" }),
    navigation.getByRole("button", { name: "Note actions" }).first(),
    navigation.getByRole("button", { name: "Filter by tag product" }),
    navigation.getByRole("button", { name: "Filter by tag writing" }),
    page.getByRole("button", { name: "Export backup" }),
    page.locator('label[aria-label="Import backup"]'),
  ];

  for (const target of targets) {
    const box = await target.evaluate((element) => element.getBoundingClientRect().toJSON());
    expect(Math.min(box.width, box.height)).toBeGreaterThanOrEqual(44);
  }

  const missingBackupDotColor = await page
    .getByText("No backup yet")
    .locator("xpath=preceding-sibling::span[1]")
    .evaluate((element) => getComputedStyle(element).backgroundColor);
  expect(missingBackupDotColor).not.toBe("rgb(16, 185, 129)");
});

test("Markdown syntax modal keeps dense reference content accessible", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedNotes(page);

  await page.getByText("Architecture note", { exact: true }).click();
  await page.getByRole("button", { name: "Show markdown cheatsheet" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading", { name: "Markdown syntax" })).toBeVisible();
  const dialogBox = await dialog.evaluate((element) =>
    element.getBoundingClientRect().toJSON(),
  );
  expect(dialogBox.width).toBeLessThan(page.viewportSize().width);
  expect(dialogBox.height).toBeLessThan(page.viewportSize().height);
  expect(dialogBox.left).toBeGreaterThan(0);
  await expect(dialog.locator("xpath=parent::*")).not.toHaveAttribute("role", "dialog");

  const closeBox = await dialog
    .getByRole("button", { name: "Close markdown syntax" })
    .evaluate((element) => element.getBoundingClientRect().toJSON());
  expect(closeBox.width).toBeGreaterThanOrEqual(44);
  expect(closeBox.height).toBeGreaterThanOrEqual(44);

  const sections = await dialog.locator("h3,h4").evaluateAll((headings) =>
    headings.map((heading) => heading.textContent.trim()),
  );
  expect(sections.slice(0, 3)).toEqual(["Core syntax", "Headings", "Emphasis"]);
  expect(sections).toContain("Extended syntax");

  const bodyMetrics = await dialog.locator(".overflow-y-auto").evaluate((element) => ({
    clientHeight: element.clientHeight,
    scrollHeight: element.scrollHeight,
  }));
  expect(bodyMetrics.clientHeight).toBeGreaterThan(300);
  expect(bodyMetrics.scrollHeight).toBeGreaterThan(bodyMetrics.clientHeight);
});

test("About Nota uses the main-frame popup model like Markdown help", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedNotes(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  const aboutButton = navigation.getByRole("button", { name: "About Nota" });
  await aboutButton.click();

  const dialog = page.getByRole("dialog", { name: "About Nota" });
  await expect(dialog).toHaveAttribute("aria-labelledby", "about-modal-title");
  await expect(dialog).toHaveAttribute("aria-describedby", "about-modal-description");
  await expect(dialog).toContainText("Local browser storage");
  await expect(dialog).toContainText("Backup Health");

  const isNestedInSidebar = await dialog.evaluate((element) =>
    Boolean(element.closest('nav[aria-label="Notes sidebar"]')),
  );
  expect(isNestedInSidebar).toBe(false);
  await expect(dialog.locator("xpath=parent::*")).not.toHaveAttribute("role", "dialog");

  const closeBox = await dialog
    .getByRole("button", { name: "Close About Nota" })
    .evaluate((element) => element.getBoundingClientRect().toJSON());
  expect(closeBox.width).toBeGreaterThanOrEqual(44);
  expect(closeBox.height).toBeGreaterThanOrEqual(44);

  const footerClose = dialog.getByRole("button", { name: "Close", exact: true });
  const footerCloseColor = await footerClose.evaluate((element) =>
    getComputedStyle(element).backgroundColor,
  );
  expect(footerCloseColor).toBe("rgb(255, 179, 64)");

  await footerClose.click();
  await expect(aboutButton).toBeFocused();
});

test("Search Hint appears as a popup without shifting the Note List", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await seedNotes(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  const firstNote = navigation
    .getByText("Architecture note", { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'group')][1]");
  const topBefore = await firstNote.evaluate((element) => element.getBoundingClientRect().top);

  await page.getByPlaceholder("Search").focus();
  await expect(page.getByText("Syntax")).toBeVisible();

  const topAfter = await firstNote.evaluate((element) => element.getBoundingClientRect().top);
  expect(Math.abs(topAfter - topBefore)).toBeLessThanOrEqual(1);
});

test("Light and Dark Theme keep core visual contracts readable", async ({ page }) => {
  await seedNotes(page);

  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  await expect(navigation.getByText("Architecture note", { exact: true })).toBeVisible();
  await expect(page.getByPlaceholder("Search")).toBeVisible();
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Architecture note");

  await page.getByPlaceholder("Search").focus();
  const searchHint = page.getByText("Syntax").locator("..");
  await expect(searchHint).toBeVisible();
  const lightHint = await visibleContrast(searchHint);
  expect(lightHint.color).toBe("rgb(37, 34, 31)");
  expect(lightHint.backgroundColor).toBe("rgb(253, 252, 249)");

  const selectedRow = navigation
    .getByText("Architecture note", { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'group')][1]");
  const selectedStyle = await visibleContrast(selectedRow);
  expect(selectedStyle.borderColor).not.toBe("rgba(0, 0, 0, 0)");
  expect(selectedStyle.borderColor).not.toBe("rgb(229, 231, 235)");

  await page.getByRole("button", { name: "Switch to dark mode" }).click();
  await page.getByPlaceholder("Search").focus();
  const darkHint = await visibleContrast(searchHint);
  expect(darkHint.color).toBe("rgb(247, 245, 241)");
  expect(darkHint.backgroundColor).not.toBe("rgba(0, 0, 0, 0)");

  for (const mode of ["Preview", "Split"]) {
    await page.getByRole("button", { name: `${mode} mode` }).click();
    const previewColors = await page.locator("article.prose").first().evaluate((article) => {
      const pane = article.parentElement;
      return {
        paneBackground: getComputedStyle(pane).backgroundColor,
        title: getComputedStyle(article.querySelector("h1")).color,
        body: getComputedStyle(article.querySelector("p")).color,
      };
    });

    expect(previewColors.paneBackground).not.toBe("rgb(255, 255, 255)");
    expect(previewColors.title).toBe("rgb(247, 245, 241)");
    expect(previewColors.body).toBe("rgb(209, 213, 219)");
  }
});

test("main app frame uses the Quiet Notebook material system", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await seedNotes(page);

  const frame = page.locator('[data-testid="app-frame"]');
  const workspace = page.locator('[data-testid="workspace-frame"]');
  const navigation = page.getByRole("navigation", { name: "Notes sidebar" });
  const editorPane = page.locator('[data-testid="editor-frame"]').first();
  const title = page.getByPlaceholder("Note Title");
  const selectedRow = navigation
    .getByText("Architecture note", { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'group')][1]");

  const lightFrame = await frame.evaluate((element) => {
    const styles = getComputedStyle(element);
    return {
      backgroundColor: styles.backgroundColor,
      paddingTop: styles.paddingTop,
      paddingRight: styles.paddingRight,
    };
  });
  expect(lightFrame.backgroundColor).toBe("rgb(247, 245, 241)");
  expect(lightFrame.paddingTop).toBe("12px");
  expect(lightFrame.paddingRight).toBe("12px");

  const lightWorkspace = await workspace.evaluate((element) => {
    const styles = getComputedStyle(element);
    return {
      borderColor: styles.borderColor,
      borderTopWidth: styles.borderTopWidth,
      borderRadius: styles.borderTopLeftRadius,
    };
  });
  expect(lightWorkspace).toEqual({
    borderColor: "rgb(230, 226, 218)",
    borderTopWidth: "1px",
    borderRadius: "8px",
  });

  const frameBoxes = await Promise.all([
    workspace.evaluate((element) => element.getBoundingClientRect().toJSON()),
    navigation.evaluate((element) => element.getBoundingClientRect().toJSON()),
    editorPane.evaluate((element) => element.getBoundingClientRect().toJSON()),
  ]);
  const [workspaceBox, navigationBox, editorBox] = frameBoxes;
  expect(navigationBox.top).toBe(editorBox.top);
  expect(navigationBox.bottom).toBe(editorBox.bottom);
  expect(navigationBox.top).toBe(workspaceBox.top + 1);
  expect(editorBox.bottom).toBe(workspaceBox.bottom - 1);

  const lightSurfaces = await Promise.all([
    navigation.evaluate((element) => {
      const styles = getComputedStyle(element);
      return {
        backgroundColor: styles.backgroundColor,
        borderColor: styles.borderRightColor,
      };
    }),
    editorPane.evaluate((element) => {
      const styles = getComputedStyle(element);
      return {
        backgroundColor: styles.backgroundColor,
        borderColor: styles.borderColor,
      };
    }),
    title.evaluate((element) => {
      const styles = getComputedStyle(element);
      return {
        backgroundColor: styles.backgroundColor,
        color: styles.color,
      };
    }),
    selectedRow.evaluate((element) => {
      const styles = getComputedStyle(element);
      return {
        backgroundColor: styles.backgroundColor,
        borderColor: styles.borderColor,
      };
    }),
  ]);

  expect(lightSurfaces[0]).toEqual({
    backgroundColor: "rgb(240, 237, 230)",
    borderColor: "rgb(216, 209, 197)",
  });
  expect(lightSurfaces[1]).toEqual({
    backgroundColor: "rgb(253, 252, 249)",
    borderColor: "rgb(230, 226, 218)",
  });
  expect(lightSurfaces[2].backgroundColor).toBe("rgba(0, 0, 0, 0)");
  expect(lightSurfaces[2].color).toBe("rgb(37, 34, 31)");
  expect(lightSurfaces[3].backgroundColor).toBe("rgb(242, 224, 190)");
  expect(lightSurfaces[3].borderColor).toBe("rgb(219, 167, 86)");

  await seedNotes(page, { dark: true });

  const darkFrame = await frame.evaluate((element) => getComputedStyle(element).backgroundColor);
  const darkEditor = await editorPane.evaluate((element) => {
    const styles = getComputedStyle(element);
    return {
      backgroundColor: styles.backgroundColor,
      borderColor: styles.borderColor,
    };
  });

  expect(darkFrame).toBe("rgb(21, 19, 17)");
  expect(darkEditor).toEqual({
    backgroundColor: "rgb(37, 34, 31)",
    borderColor: "rgb(58, 55, 51)",
  });
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
    page
      .getByRole("navigation", { name: "Notes sidebar" })
      .locator("h3")
      .getByText("New Note", { exact: true }),
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
  await expect(editorFooter.getByText(/^Lines: \d+$/)).toBeVisible();
  await expect(editorFooter.getByText(/^Words: \d+$/)).toBeVisible();
  await expect(editorFooter.getByText(/^Characters: \d+$/)).toBeVisible();
  await expect(editorFooter.getByText("Mode")).toBeVisible();
  await expect(editorFooter.getByText("1:1")).toHaveCount(0);
  await expect(editorFooter.getByRole("button", { name: "Focus editor" })).toHaveCount(0);

  const writeTitleScale = await page
    .getByPlaceholder("Note Title")
    .evaluate((element) => parseFloat(getComputedStyle(element).fontSize));
  const writePaneLayout = await page.getByPlaceholder("Note Title").evaluate((element) => {
    const pane = element.closest(".flex-1.flex.flex-col");
    const paneBox = pane.getBoundingClientRect();
    const titleBox = element.getBoundingClientRect();
    return {
      leftInset: titleBox.left - paneBox.left,
      topInset: titleBox.top - paneBox.top,
    };
  });

  await page.getByRole("button", { name: "Preview mode" }).click();
  const fullPreviewLayout = await page.locator("article.prose").first().evaluate((article) => {
    const pane = article.parentElement;
    const paneBox = pane.getBoundingClientRect();
    const articleBox = article.getBoundingClientRect();
    return {
      leftInset: articleBox.left - paneBox.left,
      topInset: articleBox.top - paneBox.top,
    };
  });
  expect(Math.abs(fullPreviewLayout.leftInset - writePaneLayout.leftInset)).toBeLessThanOrEqual(1);
  expect(Math.abs(fullPreviewLayout.topInset - writePaneLayout.topInset)).toBeLessThanOrEqual(1);

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

  const splitPreviewLayout = await page.locator("article.prose").first().evaluate((article) => {
    const pane = article.parentElement;
    return {
      leftInset: article.getBoundingClientRect().left - pane.getBoundingClientRect().left,
      topInset: article.getBoundingClientRect().top - pane.getBoundingClientRect().top,
    };
  });
  expect(Math.abs(splitPreviewLayout.leftInset - writePaneLayout.leftInset)).toBeLessThanOrEqual(1);
  expect(Math.abs(splitPreviewLayout.topInset - writePaneLayout.topInset)).toBeLessThanOrEqual(1);

  const splitTitleScale = await page.evaluate(() => {
    const editorTitle = document.querySelector('textarea[placeholder="Note Title"]');
    const previewTitle = document.querySelector(".prose > h1");
    return {
      editor: parseFloat(getComputedStyle(editorTitle).fontSize),
      preview: parseFloat(getComputedStyle(previewTitle).fontSize),
    };
  });
  expect(splitTitleScale.editor).toBe(writeTitleScale);
  expect(splitTitleScale.preview).toBeGreaterThanOrEqual(splitTitleScale.editor);

  const body = page.getByPlaceholder("Start typing...");
  await body.fill(`${note.content}\n\nSaved by browser coverage.`);
  const notification = page.getByRole("status");
  await expect(notification).toContainText(/Saving|Saved/);

  const notificationBox = await notification.evaluate((element) =>
    element.getBoundingClientRect().toJSON(),
  );
  const viewport = page.viewportSize();
  expect(notificationBox.top).toBeGreaterThanOrEqual(20);
  expect(viewport.width - notificationBox.right).toBeGreaterThanOrEqual(20);

  await expect(notification).toBeHidden({ timeout: 5_000 });
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
    page.getByRole("button", { name: "Edit tags" }),
    page.getByRole("button", { name: "Bold" }),
    page.getByRole("button", { name: "Italic" }),
    page.getByRole("button", { name: "Strikethrough" }),
    page.getByRole("button", { name: "Task list" }),
    page.getByRole("button", { name: "Insert table" }),
    page.getByRole("button", { name: "Write mode" }),
    page.getByRole("button", { name: "Preview mode" }),
    page.getByRole("button", { name: "Show markdown cheatsheet" }),
  ];

  for (const target of mobileTouchTargets) {
    const box = await target.evaluate((element) => element.getBoundingClientRect().toJSON());
    expect(Math.min(box.width, box.height)).toBeGreaterThanOrEqual(44);
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
  await page.setViewportSize({ width: 1920, height: 1080 });
  await seedNotes(page);

  const body = page.getByPlaceholder("Start typing...");
  const measure = await body.evaluate((element) => {
    const title = document.querySelector('textarea[placeholder="Note Title"]').closest(".space-y-2");
    const toolbar = Array.from(document.querySelectorAll("div")).find(
      (candidate) =>
        candidate.className.includes("max-w-[72ch]") &&
        candidate.textContent.includes("B") &&
        candidate.textContent.includes("I"),
    );
    const styles = getComputedStyle(element);
    const titleBox = title.getBoundingClientRect();
    const toolbarBox = toolbar.getBoundingClientRect();
    const bodyBox = element.getBoundingClientRect();
    return {
      titleLeft: titleBox.left,
      toolbarLeft: toolbarBox.left,
      bodyLeft: bodyBox.left,
      width: bodyBox.width,
      maxWidth: styles.maxWidth,
    };
  });

  expect(measure.width).toBeLessThanOrEqual(920);
  expect(measure.maxWidth).not.toBe("none");
  expect(Math.abs(measure.bodyLeft - measure.titleLeft)).toBeLessThanOrEqual(1);
  expect(Math.abs(measure.bodyLeft - measure.toolbarLeft)).toBeLessThanOrEqual(1);
});
