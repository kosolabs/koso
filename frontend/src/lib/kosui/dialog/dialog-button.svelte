<script module lang="ts">
  import type { Snippet } from "svelte";
  import { Button, type ButtonVariants } from "../button";

  type ButtonProps<T> = {
    value: T;
    autofocus?: boolean;
    onSelect: (value: T) => void;
    children: Snippet;
  } & ButtonVariants;
</script>

<script lang="ts" generics="T">
  let {
    value,
    variant,
    onSelect,
    autofocus = false,
    children,
  }: ButtonProps<T> = $props();
</script>

<Button {variant} onclick={() => onSelect(value)} {autofocus}>
  {@render children()}
</Button>
