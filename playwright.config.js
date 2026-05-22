const { defineConfig, devices } = require("@playwright/test");

module.exports = defineConfig({
  testDir: "./tests/browser",
  workers: 1,
  timeout: 30_000,
  expect: {
    timeout: 5_000,
  },
  webServer: {
    command: "env -u NO_COLOR trunk serve --port 1420 --no-autoreload",
    url: "http://127.0.0.1:1420",
    reuseExistingServer: false,
    timeout: 120_000,
  },
  use: {
    baseURL: "http://127.0.0.1:1420",
    trace: "retain-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"], viewport: { width: 1280, height: 900 } },
    },
  ],
});
