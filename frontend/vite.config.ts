import { sveltekit } from "@sveltejs/kit/vite";
import { existsSync, lstatSync, readlinkSync } from "fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { join } from "path";
import { defineConfig } from "vitest/config";

// Check if kosui is linked (symlinked)
// To link kosui, in the kosui repo run (just once):
//   pnpm link
// In this repo, run:
//   pnpm link kosui && pnpm install
// To unlink kosui, run:
//   pnpm unlink kosui && pnpm install
const kosuiPath = join(process.cwd(), "node_modules/kosui");
const isKosuiLinked =
  existsSync(kosuiPath) &&
  lstatSync(kosuiPath).isSymbolicLink() &&
  !readlinkSync(kosuiPath).startsWith(".pnpm/");

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
      "/plugins": {
        target: "http://localhost:3000",
        ws: true,
        changeOrigin: true,
        secure: false,
      },
      "/oauth": {
        target: "http://localhost:3000",
        ws: true,
        changeOrigin: true,
        secure: false,
      },
      "/.well-known/oauth-protected-resource": {
        target: "http://localhost:3000",
        ws: true,
        changeOrigin: true,
        secure: false,
      },
      "/.well-known/oauth-authorization-server": {
        target: "http://localhost:3000",
        ws: true,
        changeOrigin: true,
        secure: false,
      },
    },
    ...(isKosuiLinked && {
      watch: {
        ignored: ["!**/node_modules/kosui/**"],
      },
    }),
  },
  ...(isKosuiLinked && {
    optimizeDeps: {
      exclude: ["kosui"],
    },
  }),
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
    setupFiles: "./tests/setup.ts",
    environment: "jsdom",
    execArgv: [
      "--localstorage-file",
      path.resolve(os.tmpdir(), `vitest-${process.pid}.localstorage`),
    ],
  },
  resolve: process.env.VITEST
    ? {
        conditions: ["browser"],
      }
    : undefined,
});
