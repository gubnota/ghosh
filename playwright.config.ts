import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests/ui", fullyParallel: false, workers: 1,
  use: { baseURL: "http://127.0.0.1:1420", viewport: { width: 1160, height: 780 }, trace: "retain-on-failure" },
  webServer: { command: "npm run dev", url: "http://127.0.0.1:1420", reuseExistingServer: false },
});
