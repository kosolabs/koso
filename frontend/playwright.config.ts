import type { PlaywrightTestConfig } from "@playwright/test";

const port = process.env.CI ? 3000 : process.env.PW_SERVER_PORT || 5173;

const config: PlaywrightTestConfig = {
  webServer: {
    command: process.env.KOSO_IMAGE
      ? `docker run \
        --env KOSO_ENV=dev \
        --env KOSO_SETTING_DATABASE_URL \
        --env KOSO_SETTING_HOST=localhost:${port} \
        --env RUST_LOG=info \
        -v ${process.env.KOSO_SETTING_SECRETS_DIR}:/.secrets \
        --network=host \
        --rm ${process.env.KOSO_IMAGE}`
      : "pnpm run build && (cd ../backend && ./scripts/run_dev.sh)",
    port,
    reuseExistingServer: !process.env.CI,
    stdout: "pipe",
    timeout: 180000,
  },
  testDir: "tests",
  testMatch: /(.+\.)?(test|spec)\.[jt]s/,
  fullyParallel: true,
};

export default config;
