<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { Variants } from "../base";
  import { Button } from "../button";

  type ButtonProps<T> = {
    value: T;
    autofocus?: boolean;
    onSelect: (value: T) => void;
    children: Snippet;
  } & Variants;
</script>

<script lang="ts" generics="T">
  let {
    value,
    onSelect,
    autofocus = false,
    children,
    ...restProps
  }: ButtonProps<T> = $props();
</script>

<Button onclick={() => onSelect(value)} {autofocus} {...restProps}>
  {@render children()}
</Button>
