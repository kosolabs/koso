import { crossfade, slide } from "svelte/transition";

export const [send, receive] = crossfade({
  duration: (d) => Math.sqrt(d * 250),
  fallback(node) {
    return slide(node);
  },
});
