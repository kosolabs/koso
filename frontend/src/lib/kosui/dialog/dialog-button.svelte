<script module lang="ts">
  import type { Snippet } from "svelte";
  import { Button, type ButtonProps } from "../button";

  export type DialogButtonProps<T> = {
    value: T | undefined;
    autofocus?: boolean;
    onSelect: (value: T | undefined) => void;
    children: Snippet;
  } & Omit<ButtonProps, "value">;
</script>

<script lang="ts" generics="T">
  let {
    value,
    onSelect,
    autofocus = false,
    children,
    ...restProps
  }: DialogButtonProps<T> = $props();
</script>

<Button onclick={() => onSelect(value)} {autofocus} {...restProps}>
  {@render children()}
</Button>
