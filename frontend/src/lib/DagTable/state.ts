import { derived, writable } from "svelte/store";
import type { Node } from "../koso";

export const nodes = writable<Node[]>([]);

export const selected = writable<Node | null>(null);
export const highlighted = writable<Node | null>(null);
export const dropEffect = writable<"link" | "move" | "none">("none");
export const dragged = writable<Node | null>(null);
export const collapsed = writable<Set<string>>(new Set());

export const hidden = derived(
  [nodes, collapsed],
  ([$nodes, $collapsed]) =>
    new Set(
      $nodes
        .map((node) => node.id)
        .filter((node) => {
          for (const c of $collapsed) {
            if (node.startsWith(c + "-")) {
              return true;
            }
          }
          return false;
        }),
    ),
);
