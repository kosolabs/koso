import { Set } from "immutable";
import type { SvelteComponent } from "svelte";

let popovers = Set<SvelteComponent>();

export function globalKeybindingsEnabled(): boolean {
  return popovers.size === 0;
}

export function handleOpenChange(
  open: boolean,
  popover: SvelteComponent | undefined,
) {
  if (!popover) return;
  if (open) {
    popovers = popovers.add(popover);
  } else {
    popovers = popovers.remove(popover);
  }
}
