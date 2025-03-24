<script lang="ts" module>
  import { type Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { Variants } from "../base";
  import { mergeProps } from "../merge-props";
  import { Shortcut } from "../shortcut";
  import { type ClassName } from "../utils";
  import { getMenuContext } from "./menu-context.svelte";

  export type MenuTriggerProps = {
    children?: Snippet;
  } & ClassName &
    Variants &
    HTMLButtonAttributes;
</script>

<script lang="ts">
  let { children, class: className, ...restProps }: MenuTriggerProps = $props();

  const ctx = getMenuContext();
</script>

<button
  bind:this={ctx.anchorEl}
  class={twMerge(className)}
  {...mergeProps(
    {
      onclick: () => (ctx.open = !ctx.open),
      onkeydown: (event: KeyboardEvent) => {
        if (Shortcut.ENTER.matches(event) || Shortcut.SPACE.matches(event)) {
          event.stopImmediatePropagation();
        }
      },
      "aria-haspopup": "menu",
      "aria-expanded": ctx.open,
    },
    restProps,
  )}
>
  {@render children?.()}
</button>
