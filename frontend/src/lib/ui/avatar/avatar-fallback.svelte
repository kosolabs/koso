<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";

  type Props = HTMLAttributes<HTMLSpanElement>;

  let { children, class: classes, ...props }: Props = $props();
</script>

<span
  class={cn("flex h-full w-full items-center justify-center bg-muted", classes)}
  {...props}
>
  {#if children}
    {@render children()}
  {/if}
</span>
