<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { baseClasses, type Variants } from "../base";
  import { Popover, type PopoverProps } from "../popover";
  import type { ClassName } from "../utils";
  import { getAutocompleteContext } from "./autocomplete-context.svelte";

  export type AutocompleteProps = {
    children: Snippet;
  } & ClassName &
    Variants &
    Omit<PopoverProps, "children">;
</script>

<script lang="ts">
  let {
    open = $bindable(false),
    children,
    class: className,
    variant = "elevated",
    color = "primary",
    shape = "rounded",
    ...restProps
  }: AutocompleteProps = $props();

  const ctx = getAutocompleteContext();
  ctx.bindOpen(
    () => open,
    (newval) => (open = newval),
  );
</script>

<Popover
  bind:open={ctx.open}
  role="menu"
  class={twMerge(
    baseClasses({ variant, color, shape }),
    "bg-m3-surface-container-highest max-h-[40%] border p-1 shadow",
    className,
  )}
  anchorEl={ctx.anchorEl}
  placement="bottom"
  {...restProps}
>
  {@render children()}
</Popover>
