<script lang="ts">
  import { Button } from "$lib/ui/button";
  import { Tooltip } from "$lib/ui/tooltip";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Props = HTMLButtonAttributes & {
    title: string;
    Icon: any;
  };

  let { children, title, Icon, class: classes, ...props }: Props = $props();
</script>

<Tooltip>
  {#snippet trigger()}
    <Button class={classes} variant="ghost" {...props}>
      <Icon class="w-4 sm:me-2" />
      <div class="text-xs max-sm:hidden">{title}</div>
    </Button>
  {/snippet}
  {#snippet children()}
    {title}
  {/snippet}
</Tooltip>
