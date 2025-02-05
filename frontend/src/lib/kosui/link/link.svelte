<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes } from "svelte/elements";
  import { cn } from "../utils";

  type Props = HTMLAnchorAttributes & {
    children: Snippet;
  };
  const { children, class: className, ...props }: Props = $props();
</script>

<a
  class={cn(
    "focus-visible:ring-ring text-primary inline-flex cursor-pointer items-center justify-center gap-1 rounded-md underline-offset-4 transition-colors hover:underline focus-visible:ring-1 focus-visible:outline-hidden",
    className,
  )}
  {...props}
>
  {@render children()}
</a>
