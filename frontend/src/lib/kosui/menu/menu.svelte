<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { ElementRef } from "../utils";
  import { newMenuContext } from "./menu-context.svelte";

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

  newMenuContext(
    () => open,
    (value) => (open = value),
    () => el,
    (value) => (el = value),
  );
</script>

{@render children()}
