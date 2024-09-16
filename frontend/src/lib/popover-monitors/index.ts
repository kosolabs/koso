import { Set } from "immutable";
import { type SvelteComponent } from "svelte";
import DialogMonitoredRoot from "./dialog-monitored-root.svelte";
import DropdownMenuMonitoredRoot from "./dropdown-menu-monitored-root.svelte";

let popovers = Set<SvelteComponent>();

function globalKeybindingsEnabled(): boolean {
  return popovers.size === 0;
}

function handleOpenChange(open: boolean, popover: SvelteComponent | undefined) {
  if (!popover) return;
  if (open) {
    popovers = popovers.add(popover);
  } else {
    popovers = popovers.remove(popover);
  }
}

export {
  DialogMonitoredRoot,
  DropdownMenuMonitoredRoot,
  globalKeybindingsEnabled,
  handleOpenChange,
};
