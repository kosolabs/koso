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

console.debug("Running service worker script");

sw.addEventListener("message", (event) => {
  if (event.data && event.data.type === "SKIP_WAITING") {
    console.debug("Calling service worker skipWaiting", event);
    sw.skipWaiting();
  } else {
    console.debug("Got service worker message", event);
  }
});

const precache_list = ["/", ...build, ...files, ...prerendered].map((s) => ({
  url: s,
  revision: version,
}));

console.debug("precacheAndRoute");
precacheAndRoute(precache_list);

console.debug("cleanupOutdatedCaches");
cleanupOutdatedCaches();

// Cache user avatar images
console.debug("registerRoute googleusercontent");
registerRoute(
  ({ url }) => url.hostname.endsWith("googleusercontent.com"),
  new StaleWhileRevalidate(),
);

// Serve the app from the precache
console.debug("registerRoute /");
registerRoute(new NavigationRoute(createHandlerBoundToURL("/")));

// Serve requests to /api from the network first, and from the cache if offline.
console.debug("registerRoute /api");
registerRoute(({ url }) => url.pathname.startsWith("/api"), new NetworkFirst());
