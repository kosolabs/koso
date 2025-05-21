import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import("@sveltejs/kit").Config} */
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
      // Configure how often in milliseconds version.json is polled to refresh the
      // $updated store used to reload pages following deployments.
      pollInterval: 15 * 1000,
    },
  },
};

export default config;
