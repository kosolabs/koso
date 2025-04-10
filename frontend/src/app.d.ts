// See https://kit.svelte.dev/docs/types#app
import type { PlanningContext } from "$lib/dag-table/planning-context.svelte";
import type { Koso } from "$lib/koso.svelte";

// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }

  interface Window {
    koso: Koso;
    planningCtx: PlanningContext;
    Y: Y;
  }
}

export {};
