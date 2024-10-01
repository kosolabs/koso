import type { KeyBinding } from "$lib/key-binding";

export { default as CommandPalette } from "./command-palette.svelte";

export type Action = {
  title: string;
  // TODO: Use Component once lucide-svelte exports a Svelte 5 compatible type
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  icon: any;
  callback: () => void;
  toolbar?: boolean;
  enabled: () => boolean;
  shortcut?: KeyBinding;
};
