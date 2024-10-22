/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />

import { build, files, prerendered, version } from "$service-worker";
import { cleanupOutdatedCaches, precacheAndRoute } from "workbox-precaching";

const precache_list = [...build, ...files, ...prerendered].map((s) => ({
  url: s,
  revision: version,
}));

console.debug("Precached: ", precache_list);

precacheAndRoute(precache_list);

cleanupOutdatedCaches();
