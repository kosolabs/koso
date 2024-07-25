import { writable } from "svelte/store";
import type { Node } from ".";

export const selected = writable<Node | null>(null);
export const highlighted = writable<Node | null>(null);
export const dropEffect = writable<"link" | "move" | "none">("none");
export const dragged = writable<Node | null>(null);
