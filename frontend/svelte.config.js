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
      pollInterval: 60 * 1000,
    },
  },
  // Force rune mode on all project svelte files to avoid any issues
  // arising from accidentally running in legacy mode.
  // See https://svelte.dev/docs/svelte/svelte-compiler#CompileOptions
  // `runes` field for details.
  vitePlugin: {
    dynamicCompileOptions({ filename, compileOptions }) {
      if (!filename.includes("node_modules") && !compileOptions.runes) {
        return { runes: true };
      }
    },
  },
};

export default config;
