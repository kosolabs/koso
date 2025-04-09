import type { YTaskProxy } from "$lib/yproxy";
import { getContext, setContext } from "svelte";

export class InboxContext {
  #selected: YTaskProxy | undefined = $state();

  get selected(): YTaskProxy | undefined {
    return this.#selected;
  }

  set selected(value: YTaskProxy | undefined) {
    this.#selected = value;
  }
}

export function newInboxContext() {
  return setInboxContext(new InboxContext());
}

export function setInboxContext(ctx: InboxContext): InboxContext {
  return setContext<InboxContext>(InboxContext, ctx);
}

export function getInboxContext(): InboxContext {
  return getContext<InboxContext>(InboxContext);
}
