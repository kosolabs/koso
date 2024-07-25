import { writable } from "svelte/store";
import type { Node } from ".";

export const selected = writable<Node | null>(null);
