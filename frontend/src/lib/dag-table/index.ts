import type { Status, YTaskProxy } from "$lib/yproxy";
import type { Koso } from "./koso.svelte";

export { Koso, Node } from "./koso.svelte";
export { KosoSocket } from "./socket.svelte";
export { default as DagTable } from "./table.svelte";

/** Compares two tasks by status and number of children. */
export function compareTasks(
  t1: YTaskProxy,
  t2: YTaskProxy,
  koso: Koso,
): number {
  function mapStatus(status: Status) {
    switch (status) {
      case "In Progress":
        return 0;
      case "Not Started":
        return 1;
      case "Done":
        return 2;
    }
  }

  const status1 = mapStatus(koso.getStatus(t1.id));
  const status2 = mapStatus(koso.getStatus(t2.id));
  if (status1 !== status2) {
    return status1 - status2;
  }

  return t2.children.length - t1.children.length;
}
