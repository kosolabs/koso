// See https://kit.svelte.dev/docs/types#app
import type { Koso } from "$lib/koso";

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
  }
}

export {};
