<script lang="ts">
  import type { ClassName } from "$lib/kosui/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";

  type Props = {
    children: Snippet;
  } & ClassName &
    HTMLAttributes<HTMLDivElement>;
  let { children, class: className, ...restProps }: Props = $props();
</script>

<div
  class={twMerge("text-sm whitespace-nowrap max-md:hidden", className)}
  {...restProps}
>
  {@render children()}
</div>
