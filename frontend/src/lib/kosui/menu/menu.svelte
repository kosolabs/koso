<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { ElementRef } from "../utils";
  import { MenuContext, setMenuContext } from "./menu-context.svelte";

  export type MenuProps = {
    open?: boolean;
    children: Snippet;
  } & ElementRef;
</script>

<script lang="ts">
  let {
    open = $bindable(false),
    children,
    el = $bindable(),
  }: MenuProps = $props();

  setMenuContext(
    new MenuContext(
      () => open,
      (value) => (open = value),
      () => el,
      (value) => (el = value),
    ),
  );
</script>

{@render children()}
