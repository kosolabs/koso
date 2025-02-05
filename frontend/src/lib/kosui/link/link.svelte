<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes } from "svelte/elements";

  type Props = HTMLAnchorAttributes & {
    children: Snippet;
  };
  const { children, class: className, ...props }: Props = $props();
</script>

<a
  class={cn("text-primary underline-offset-4 hover:underline", className)}
  {...props}
>
  {@render children()}
</a>
