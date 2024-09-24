<script lang="ts">
  import { Button } from "$lib/ui/button";
  import { Tooltip } from "$lib/ui/tooltip";
  import { cn } from "$lib/utils";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Props = HTMLButtonAttributes & {
    title: string;
    Icon: any;
  };

  let { children, title, Icon, class: classes, ...props }: Props = $props();
</script>

<Tooltip
  class={cn("rounded bg-primary px-3 py-1 text-sm text-primary-foreground")}
  portalEl={document.body}
>
  {#snippet trigger({ show, hide })}
    <Button
      class={classes}
      variant="ghost"
      onmouseenter={show}
      onmouseleave={hide}
      onfocus={show}
      onblur={hide}
      {...props}
    >
      <Icon class="w-4 sm:me-2" />
      <div class="text-xs max-sm:hidden">{title}</div>
    </Button>
  {/snippet}
  {#snippet children()}
    {title}
  {/snippet}
</Tooltip>
