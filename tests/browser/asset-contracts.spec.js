const { expect, test } = require("@playwright/test");
const { builtCss } = require("./helpers/asset-contract-helpers");

test("Tailwind build emits the critical utility classes used by visual recipes", async ({ page }) => {
  await page.goto("/");

  const css = await builtCss(page);

  expect(css).toContain(".h-\\[45px\\]");
  expect(css).toContain(".min-w-\\[2\\.75rem\\]");
  expect(css).toContain(".bg-apple-notebook-selected");
  expect(css).toContain(".bg-apple-notebook-muted\\/60");
  expect(css).toContain(".dark\\:bg-apple-notebook-amber\\/25");
  expect(css).toContain(".border-apple-notebook-border");
  expect(css).toContain(".dark\\:border-apple-notebook-darkBorder");
  expect(css).toContain(".transition-transform");
  expect(css).toContain("Source Sans 3 Variable");
  expect(css).toContain("Source Code Pro Variable");
  expect(css).toContain("fonts/source-sans-3/source-sans-3-latin-wght-normal.woff2");
  expect(css).toContain("fonts/source-code-pro/source-code-pro-latin-wght-normal.woff2");
  expect(css).not.toContain(".noter-footer-height");
  expect(css).not.toContain(".min-w-9");
  expect(css).not.toContain(".transition-all");
  expect(css).not.toContain(".bg-apple-gray-100");
  expect(css).not.toContain(".text-white");
  expect(css).not.toContain(".bg-apple-yellow\\/15");
});

test("app exposes brand icon assets for browser tabs and install surfaces", async ({ page, request }) => {
  await page.goto("/");

  const headAssets = await page.evaluate(() => ({
    manifest: document.querySelector('link[rel="manifest"]')?.getAttribute("href"),
    svgIcon: document.querySelector('link[rel="icon"][type="image/svg+xml"]')?.getAttribute("href"),
    icoIcon: document.querySelector('link[rel="icon"][sizes="any"]')?.getAttribute("href"),
    appleIcon: document.querySelector('link[rel="apple-touch-icon"]')?.getAttribute("href"),
    themeColors: Array.from(document.querySelectorAll('meta[name="theme-color"]'), (meta) => ({
      media: meta.getAttribute("media"),
      content: meta.getAttribute("content"),
    })),
  }));

  expect(headAssets).toEqual({
    manifest: "assets/site.webmanifest",
    svgIcon: "assets/icons/noter-favicon.svg",
    icoIcon: "assets/icons/favicon.ico",
    appleIcon: "assets/icons/apple-touch-icon.png",
    themeColors: [
      { media: "(prefers-color-scheme: light)", content: "#F7F5F1" },
      { media: "(prefers-color-scheme: dark)", content: "#151311" },
    ],
  });

  const manifestResponse = await request.get("/assets/site.webmanifest");
  expect(manifestResponse.ok()).toBe(true);
  const manifest = await manifestResponse.json();
  expect(manifest).toMatchObject({
    name: "Nota",
    short_name: "Nota",
    display: "standalone",
    background_color: "#F7F5F1",
    theme_color: "#FFB340",
  });
  expect(manifest.icons).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ src: "icons/noter-192.png", sizes: "192x192", purpose: "any" }),
      expect.objectContaining({ src: "icons/noter-512.png", sizes: "512x512", purpose: "any" }),
      expect.objectContaining({
        src: "icons/noter-maskable-192.png",
        sizes: "192x192",
        purpose: "maskable",
      }),
      expect.objectContaining({
        src: "icons/noter-maskable-512.png",
        sizes: "512x512",
        purpose: "maskable",
      }),
    ]),
  );

  const assetPaths = [
    "/assets/icons/noter-favicon.svg",
    "/assets/icons/favicon.ico",
    "/assets/icons/noter-16.png",
    "/assets/icons/noter-32.png",
    "/assets/icons/apple-touch-icon.png",
    "/assets/icons/noter-192.png",
    "/assets/icons/noter-512.png",
    "/assets/icons/noter-maskable-192.png",
    "/assets/icons/noter-maskable-512.png",
  ];

  for (const path of assetPaths) {
    const response = await request.get(path);
    expect(response.ok()).toBe(true);
    expect((await response.body()).length).toBeGreaterThan(100);
  }

  const faviconSvg = await (await request.get("/assets/icons/noter-favicon.svg")).text();
  expect(faviconSvg).toContain("Canvas-filling folded note mark");
  expect(faviconSvg).not.toContain('<rect width="64" height="64"');

  const pngSizes = await page.evaluate(async () => {
    const sources = [
      "/assets/icons/noter-16.png",
      "/assets/icons/noter-32.png",
      "/assets/icons/apple-touch-icon.png",
      "/assets/icons/noter-192.png",
      "/assets/icons/noter-512.png",
      "/assets/icons/noter-maskable-192.png",
      "/assets/icons/noter-maskable-512.png",
    ];

    return Object.fromEntries(
      await Promise.all(
        sources.map(
          (src) =>
            new Promise((resolve, reject) => {
              const image = new Image();
              image.onload = () => resolve([src, { width: image.naturalWidth, height: image.naturalHeight }]);
              image.onerror = () => reject(new Error(`Could not load ${src}`));
              image.src = src;
            }),
        ),
      ),
    );
  });
  expect(pngSizes).toEqual({
    "/assets/icons/noter-16.png": { width: 16, height: 16 },
    "/assets/icons/noter-32.png": { width: 32, height: 32 },
    "/assets/icons/apple-touch-icon.png": { width: 180, height: 180 },
    "/assets/icons/noter-192.png": { width: 192, height: 192 },
    "/assets/icons/noter-512.png": { width: 512, height: 512 },
    "/assets/icons/noter-maskable-192.png": { width: 192, height: 192 },
    "/assets/icons/noter-maskable-512.png": { width: 512, height: 512 },
  });

  const faviconAlphaBounds = await page.evaluate(async () => {
    const image = new Image();
    await new Promise((resolve, reject) => {
      image.onload = resolve;
      image.onerror = () => reject(new Error("Could not load 16px favicon"));
      image.src = "/assets/icons/noter-16.png";
    });
    const canvas = document.createElement("canvas");
    canvas.width = 16;
    canvas.height = 16;
    const context = canvas.getContext("2d");
    context.drawImage(image, 0, 0);
    const data = context.getImageData(0, 0, 16, 16).data;
    const bounds = { left: 16, top: 16, right: -1, bottom: -1 };

    for (let y = 0; y < 16; y += 1) {
      for (let x = 0; x < 16; x += 1) {
        const alpha = data[(y * 16 + x) * 4 + 3];
        if (alpha > 0) {
          bounds.left = Math.min(bounds.left, x);
          bounds.top = Math.min(bounds.top, y);
          bounds.right = Math.max(bounds.right, x);
          bounds.bottom = Math.max(bounds.bottom, y);
        }
      }
    }

    return bounds;
  });
  expect(faviconAlphaBounds).toEqual({ left: 0, top: 0, right: 15, bottom: 15 });
});
