const { expect, test } = require("@playwright/test");

const notes = [
  {
    id: "11111111-1111-4111-8111-111111111111",
    title: "Migration risks checklist",
    content: "Title match already explains this row.",
    created: "2026-05-01T09:00:00Z",
    last_modified: "2026-05-01T10:00:00Z",
    tags: ["Work", "Planning"],
    is_pinned: false,
  },
  {
    id: "22222222-2222-4222-8222-222222222222",
    title: "Operations Brief",
    content:
      "Weekly release handoff with context that stays generic.\nThe compact Note List should surface migration risks before launch without opening the Note.",
    created: "2026-05-02T09:00:00Z",
    last_modified: "2026-05-02T10:00:00Z",
    tags: ["Launch", "Ops"],
    is_pinned: false,
  },
  {
    id: "33333333-3333-4333-8333-333333333333",
    title: "Field report",
    content: "Offline usage notes for travel.",
    created: "2026-05-03T09:00:00Z",
    last_modified: "2026-05-03T10:00:00Z",
    tags: ["Mobile", "Field"],
    is_pinned: false,
  },
  {
    id: "44444444-4444-4444-8444-444444444444",
    title: "Reference archive",
    content: "Background material for later review.",
    created: "2026-05-04T09:00:00Z",
    last_modified: "2026-05-04T10:00:00Z",
    tags: ["Reference"],
    is_pinned: false,
  },
];

async function seedCollection(page, { sidebarOpen = true } = {}) {
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

function navigation(page) {
  return page.getByRole("navigation", { name: "Notes sidebar" });
}

function noteRow(page, title) {
  return navigation(page)
    .getByText(title, { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'group')][1]");
}

async function renderedNoteRows(page) {
  return navigation(page).evaluate((sidebar) =>
    Array.from(sidebar.querySelectorAll("div.group"))
      .filter((element) => element.querySelector("h3"))
      .map((element) => element.getBoundingClientRect().toJSON()),
  );
}

test("compact Search results explain title and body matches without crowding the Note List", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedCollection(page);

  await page.getByPlaceholder("Search").fill("migration risks");
  await expect(navigation(page).getByText("2 matches for search: migration risks")).toBeVisible();

  const titleMatch = noteRow(page, "Migration risks checklist");
  await expect(titleMatch.locator("mark").getByText("Migration risks")).toBeVisible();
  await expect(titleMatch.getByText("Title match already explains this row.")).toBeVisible();

  const bodyMatch = noteRow(page, "Operations Brief");
  await expect(bodyMatch.getByText("migration risks")).toBeVisible();
  await expect(bodyMatch.locator("mark").getByText("migration risks")).toBeVisible();
  await expect(bodyMatch.getByText("Weekly release handoff with context")).toBeHidden();
  await expect(bodyMatch.getByText(/Matched in body/i)).toBeHidden();

  const rowBoxes = await renderedNoteRows(page);
  expect(rowBoxes).toHaveLength(2);
  expect(Math.max(...rowBoxes.map((box) => box.height))).toBeLessThanOrEqual(128);
  expect(rowBoxes[0].bottom).toBeLessThanOrEqual(rowBoxes[1].top + 1);
  for (const box of rowBoxes) {
    expect(box.height).toBeGreaterThanOrEqual(44);
  }
});

test("compact Tag matches, result status, empty state, and navigation stay usable", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await seedCollection(page);

  await page.getByPlaceholder("Search").fill("mobile");
  await expect(navigation(page).getByText("1 match for search: mobile")).toBeVisible();

  const row = noteRow(page, "Field report");
  const matchedTag = row.getByRole("button", { name: "Filter by tag Mobile" });
  const unmatchedTag = row.getByRole("button", { name: "Filter by tag Field" });
  await expect(matchedTag).toBeVisible();
  await expect(unmatchedTag).toBeVisible();
  await expect(matchedTag.locator("mark").getByText("Mobile")).toBeVisible();
  await expect(unmatchedTag.locator("mark")).toHaveCount(0);

  const rowBox = await row.evaluate((element) => element.getBoundingClientRect().toJSON());
  expect(rowBox.height).toBeGreaterThanOrEqual(44);
  expect(rowBox.height).toBeLessThanOrEqual(128);

  await row.click();
  await expect(navigation(page)).toHaveClass(/-translate-x-full/);
  await expect(page.getByPlaceholder("Note Title")).toHaveValue("Field report");
  await page.getByRole("button", { name: "Toggle sidebar" }).click();
  await expect(navigation(page)).toHaveClass(/translate-x-0/);

  await page.getByPlaceholder("Search").fill("nebula");
  await expect(navigation(page).getByText("No notes match search: nebula")).toBeVisible();
  await expect(navigation(page).getByText("Try a different search term.")).toBeVisible();
});

test("desktop Search Result Explanation remains covered while compact parity is added", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await seedCollection(page);

  await page.getByPlaceholder("Search").fill("migration risks");

  await expect(navigation(page).getByText("2 matches for search: migration risks")).toBeVisible();
  await expect(
    noteRow(page, "Migration risks checklist").locator("mark").getByText("Migration risks"),
  ).toBeVisible();
  await expect(
    noteRow(page, "Operations Brief").locator("mark").getByText("migration risks"),
  ).toBeVisible();
});
