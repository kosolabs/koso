/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />

import { build, files, prerendered, version } from "$service-worker";
import { cleanupOutdatedCaches, precacheAndRoute } from "workbox-precaching";
import { registerRoute } from "workbox-routing";
import { NetworkFirst, StaleWhileRevalidate } from "workbox-strategies";

const precache_list = [...build, ...files, ...prerendered].map((s) => ({
  url: s,
  revision: version,
}));

console.debug("Precached:", precache_list);

precacheAndRoute(precache_list);

cleanupOutdatedCaches();

// Cache user avatar images
registerRoute(
  ({ url }) => url.hostname.endsWith("googleusercontent.com"),
  new StaleWhileRevalidate(),
);

// Anything that is not /api can serve from the cache first
registerRoute(
  ({ url }) => !url.pathname.startsWith("/api"),
  new StaleWhileRevalidate(),
);

// Serve requests to /api from the network first, and from the cache if offline.
registerRoute(({ url }) => url.pathname.startsWith("/api"), new NetworkFirst());
