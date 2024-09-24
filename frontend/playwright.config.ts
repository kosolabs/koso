import type { PlaywrightTestConfig } from "@playwright/test";

const config: PlaywrightTestConfig = {
  webServer: {
    command: "npm run build && (cd ../backend && ./scripts/run_dev.sh)",
    port: process.env.CI ? 3000 : process.env.PW_SERVER_PORT || 5173,
    reuseExistingServer: !process.env.CI,
    stdout: "pipe",
    timeout: 180000,
  },
  testDir: "tests",
  testMatch: /(.+\.)?(test|spec)\.[jt]s/,
};

export default config;
