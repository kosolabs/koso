<script lang="ts">
  import { PlainTooltip } from "$lib/kosui/tooltip";
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

<PlainTooltip arrow>
  {#snippet trigger(ref, props)}
    <div bind:this={ref.value} {...props} class={cn("max-w-4 min-w-4")}>
      <ManagedTaskIcon size={16} class="text-foreground" />
    </div>
  {/snippet}
  This task is managed by the {getManagedTaskName(kind)} plugin.
</PlainTooltip>
