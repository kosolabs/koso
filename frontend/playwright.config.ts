import type { PlaywrightTestConfig } from "@playwright/test";

const config: PlaywrightTestConfig = {
  webServer: {
    command: process.env.KOSO_IMAGE
      ? `docker run \
        --env DATABASE_URL \
        --env TESTONLY_ENABLE_DEV=true \
        --env RUST_LOG=info \
        --env GH_APP_ENV=dev \
        --env SECRETS_DIR=/.secrets \
        --env RUST_BACKTRACE=1 \
        -v ${process.env.SECRETS_DIR}:/.secrets \
        --network=host \
        --rm ${process.env.KOSO_IMAGE}`
      : "pnpm run build && (cd ../backend && ./scripts/run_dev.sh)",
    port: process.env.CI ? 3000 : process.env.PW_SERVER_PORT || 5173,
    reuseExistingServer: !process.env.CI,
    stdout: "pipe",
    timeout: 180000,
  },
  testDir: "tests",
  testMatch: /(.+\.)?(test|spec)\.[jt]s/,
  fullyParallel: true,
};

export default config;
