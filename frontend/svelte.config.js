import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import * as child_process from "node:child_process";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: [vitePreprocess()],

  kit: {
    adapter: adapter({
      fallback: "index.html",
      precompress: true,
    }),
    serviceWorker: {
      register: false,
    },
    version: {
      name:
        child_process.execSync("git rev-parse HEAD").toString().trim() +
        `-${Date.now()}`,
      // Configure how often in milliseconds version.json is polled to refresh the
      // $updated store used to reload pages following deployments.
      pollInterval: 60 * 1000,
    },
  },
};

export default config;
