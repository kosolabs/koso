export { default as CommandContent } from "./command-content.svelte";
export { default as CommandDivider } from "./command-divider.svelte";
export { default as CommandInput } from "./command-input.svelte";
export { default as CommandItem } from "./command-item.svelte";
export { default as CommandSearch } from "./command-search.svelte";
export { default as Command } from "./command.svelte";
export { default as Commander } from "./commander.svelte";

import {
  actions,
  call,
  close,
  register,
  show,
  unregister,
} from "./commander.svelte";

export const command = {
  show,
  close,
  register,
  unregister,
  actions,
  call,
};
