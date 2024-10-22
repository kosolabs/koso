/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />

import { build, files, prerendered, version } from "$service-worker";
import "workbox-core";
import {
  cleanupOutdatedCaches,
  createHandlerBoundToURL,
  precacheAndRoute,
} from "workbox-precaching";
import { NavigationRoute, registerRoute } from "workbox-routing";
import { NetworkFirst, StaleWhileRevalidate } from "workbox-strategies";

const sw = self as unknown as ServiceWorkerGlobalScope;
sw.__WB_DISABLE_DEV_LOGS = true;

sw.addEventListener("message", (event) => {
  if (event.data && event.data.type === "SKIP_WAITING") {
    sw.skipWaiting();
  }
});

const precache_list = ["/index.html", ...build, ...files, ...prerendered].map(
  (s) => ({
    url: s,
    revision: version,
  }),
);

precacheAndRoute(precache_list);

cleanupOutdatedCaches();

// Cache user avatar images
registerRoute(
  ({ url }) => url.hostname.endsWith("googleusercontent.com"),
  new StaleWhileRevalidate(),
);

// Serve the app from the precache
registerRoute(new NavigationRoute(createHandlerBoundToURL("/index.html")));

// Serve requests to /api from the network first, and from the cache if offline.
registerRoute(({ url }) => url.pathname.startsWith("/api"), new NetworkFirst());
