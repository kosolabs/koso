<script lang="ts" module>
  import { Button, type ButtonProps } from "../button";
  import { mergeProps } from "../merge-props";
  import { Shortcut } from "../shortcut";
  import { getMenuContext } from "./menu-context.svelte";

  export type MenuTriggerButtonProps = {} & ButtonProps;
</script>

<script lang="ts">
  let { ...restProps }: MenuTriggerButtonProps = $props();

  const ctx = getMenuContext();
</script>

<Button
  bind:el={ctx.anchorEl}
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
/>
