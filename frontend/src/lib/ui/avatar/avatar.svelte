<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";

  type Props = HTMLAttributes<HTMLDivElement>;

  let { children, class: classes, ...props }: Props = $props();
</script>

<div class={cn("flex h-10 w-10", classes)} {...props}>
  {#if children}
    {@render children()}
  {/if}
</div>
