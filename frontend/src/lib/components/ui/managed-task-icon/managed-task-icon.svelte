<script lang="ts">
  import { Tooltip } from "$lib/kosui/tooltip";
  import TooltipTrigger from "$lib/kosui/tooltip/tooltip-trigger.svelte";
  import { cn } from "$lib/utils";
  import { Github, ToyBrick, type Icon } from "lucide-svelte";

  type Props = {
    kind: string;
  };
  let { kind }: Props = $props();

  function getManagedTaskIcon(kind: string): typeof Icon {
    if (kind.startsWith("github")) {
      return Github;
    }
    console.warn(`No icon registered for kind ${kind}. Add one!`);
    return ToyBrick;
  }

  function getManagedTaskName(kind: string): string {
    if (kind.startsWith("github")) {
      return "GitHub";
    }
    return "Untitled";
  }

  let ManagedTaskIcon = getManagedTaskIcon(kind);
</script>

<Tooltip arrow>
  {#snippet trigger(props)}
    <TooltipTrigger {...props} class={cn("max-w-4 min-w-4")}>
      <ManagedTaskIcon size={16} />
    </TooltipTrigger>
  {/snippet}
  This task is managed by the {getManagedTaskName(kind)} plugin.
</Tooltip>
