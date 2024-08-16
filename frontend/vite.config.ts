import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        ws: true,
        changeOrigin: true,
        secure: false,
      },
    },
  },
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
    setupFiles: "./tests/setup.ts",
    environment: "jsdom",
  },
});
