import type { YTaskProxy } from "$lib/yproxy";
import type { Koso } from "./koso.svelte";

/** Compares two tasks by status and number of children. */
export function compareTasks(
  t1: YTaskProxy,
  t2: YTaskProxy,
  koso: Koso,
): number {
  // Order non-archived tasks ahead of archived ones.
  if (!!t1.archived !== !!t2.archived) {
    return t1.archived ? 1 : -1;
  }

  const status1 = koso.getStatusOrder(koso.getStatus(t1.id));
  const status2 = koso.getStatusOrder(koso.getStatus(t2.id));
  if (status1 !== status2) {
    return status1 - status2;
  }

  return t2.children.length - t1.children.length;
}
