import { writable } from "svelte/store";
import type { IndexedNode } from "./table.svelte";

export const selected = writable<IndexedNode | null>(null);
